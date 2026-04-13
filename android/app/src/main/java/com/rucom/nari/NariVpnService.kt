package com.rucom.nari

import android.content.Intent
import android.net.VpnService
import android.os.ParcelFileDescriptor
import android.util.Log
import java.io.DataInputStream
import java.io.DataOutputStream
import java.io.FileInputStream
import java.io.FileOutputStream
import java.net.Socket
import kotlin.concurrent.thread

class NariVpnService : VpnService() {

    private var vpnInterface: ParcelFileDescriptor? = null
    private var relaySocket: Socket? = null
    private var isRunning = false

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent?.action == "STOP") {
            stopVpn()
            return START_NOT_STICKY
        }

        if (!isRunning) {
            startVpn()
        }
        return START_STICKY
    }

    private fun startVpn() {
        Log.d("NariVpn", "Starting VPN")
        try {
            val builder = Builder()
                .addAddress("10.0.0.2", 24)
                .addRoute("0.0.0.0", 0) // Route all traffic
                .addDnsServer("8.8.8.8") // Google DNS
                .setSession("Nari Reverse Tethering")
                .setMtu(1500)

            vpnInterface = builder.establish()
            isRunning = true

            thread(name = "Nari-VpnMain") {
                vpnLoop()
            }
        } catch (e: Exception) {
            Log.e("NariVpn", "Error starting VPN", e)
            stopVpn()
        }
    }

    private fun vpnLoop() {
        try {
            // Wait for ADB reverse to be set up
            var connected = false
            while (isRunning && !connected) {
                try {
                    relaySocket = Socket("127.0.0.1", 4242)
                    connected = true
                } catch (e: Exception) {
                    Log.d("NariVpn", "Waiting for relay on 4242...")
                    Thread.sleep(1000)
                }
            }

            if (!isRunning) return

            Log.d("NariVpn", "Connected to Desktop Relay!")

            val tunDevice = vpnInterface?.fileDescriptor ?: return
            val tunIn = FileInputStream(tunDevice)
            val tunOut = FileOutputStream(tunDevice)

            val relayIn = DataInputStream(relaySocket!!.getInputStream())
            val relayOut = DataOutputStream(relaySocket!!.getOutputStream())

            // Thread to read from Tun and forward to Relay
            thread(name = "Nari-TunToRelay") {
                val buffer = ByteArray(32767)
                try {
                    while (isRunning) {
                        val length = tunIn.read(buffer)
                        if (length > 0) {
                            // Forward to desktop relay with length prefix
                            relayOut.writeInt(length)
                            relayOut.write(buffer, 0, length)
                        }
                    }
                } catch (e: Exception) {
                    Log.e("NariVpn", "TunToRelay Error", e)
                    stopVpn()
                }
            }

            // Read from Relay and write to Tun
            val packetBuf = ByteArray(32767)
            while (isRunning) {
                val length = relayIn.readInt()
                if (length > 0 && length <= packetBuf.size) {
                    relayIn.readFully(packetBuf, 0, length)
                    tunOut.write(packetBuf, 0, length)
                }
            }

        } catch (e: Exception) {
            Log.e("NariVpn", "VPN loop error", e)
            stopVpn()
        }
    }

    private fun stopVpn() {
        Log.d("NariVpn", "Stopping VPN")
        isRunning = false
        try {
            vpnInterface?.close()
        } catch (e: Exception) {}
        vpnInterface = null

        try {
            relaySocket?.close()
        } catch (e: Exception) {}
        relaySocket = null
        
        stopSelf()
    }

    override fun onDestroy() {
        super.onDestroy()
        stopVpn()
    }
}
