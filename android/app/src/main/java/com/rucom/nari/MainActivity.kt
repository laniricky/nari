package com.rucom.nari

import android.animation.AnimatorSet
import android.animation.ObjectAnimator
import android.animation.ValueAnimator
import android.app.Activity
import android.content.Intent
import android.content.res.ColorStateList
import android.graphics.Color
import android.net.VpnService
import android.os.Bundle
import android.view.View
import android.view.animation.AccelerateDecelerateInterpolator
import android.widget.ImageView
import android.widget.TextView
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity
import com.google.android.material.card.MaterialCardView

class MainActivity : AppCompatActivity() {

    private var isVpnActive = false
    private lateinit var btnToggle: MaterialCardView
    private lateinit var pulseBg: View
    private lateinit var tvStatus: TextView
    private lateinit var ivPowerIcon: ImageView

    private var pulseAnimator: AnimatorSet? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        btnToggle = findViewById(R.id.btn_toggle)
        pulseBg = findViewById(R.id.pulse_bg)
        tvStatus = findViewById(R.id.tv_status)
        ivPowerIcon = findViewById(R.id.iv_power_icon)

        btnToggle.setOnClickListener {
            if (!isVpnActive) {
                // Try to start
                val intent = VpnService.prepare(this)
                if (intent != null) {
                    startActivityForResult(intent, 0)
                } else {
                    onActivityResult(0, Activity.RESULT_OK, null)
                }
            } else {
                // Stop it
                stopVpnService()
            }
        }
    }

    override fun onActivityResult(requestCode: Int, resultCode: Int, data: Intent?) {
        super.onActivityResult(requestCode, resultCode, data)
        if (requestCode == 0 && resultCode == Activity.RESULT_OK) {
            startVpnService()
        } else {
            Toast.makeText(this, "VPN permission denied", Toast.LENGTH_SHORT).show()
            setDisconnectedUi()
        }
    }

    private fun startVpnService() {
        val intent = Intent(this, NariVpnService::class.java)
        startService(intent)
        setConnectedUi()
    }

    private fun stopVpnService() {
        val intent = Intent(this, NariVpnService::class.java)
        intent.action = "STOP"
        startService(intent)
        setDisconnectedUi()
    }

    private fun setConnectedUi() {
        isVpnActive = true
        tvStatus.text = "Active & Secure"
        tvStatus.setTextColor(Color.parseColor("#06B6D4")) // nari_accent

        ivPowerIcon.imageTintList = ColorStateList.valueOf(Color.parseColor("#06B6D4"))
        btnToggle.strokeColor = Color.parseColor("#06B6D4")

        startPulseAnimation()
    }

    private fun setDisconnectedUi() {
        isVpnActive = false
        tvStatus.text = "Tap to Connect"
        tvStatus.setTextColor(Color.parseColor("#F8FAFC")) // nari_text_primary

        ivPowerIcon.imageTintList = ColorStateList.valueOf(Color.parseColor("#94A3B8"))
        btnToggle.strokeColor = Color.parseColor("#334155")

        stopPulseAnimation()
    }

    private fun startPulseAnimation() {
        pulseBg.visibility = View.VISIBLE
        
        val scaleX = ObjectAnimator.ofFloat(pulseBg, "scaleX", 1f, 1.8f)
        val scaleY = ObjectAnimator.ofFloat(pulseBg, "scaleY", 1f, 1.8f)
        val alpha = ObjectAnimator.ofFloat(pulseBg, "alpha", 0.5f, 0f)

        scaleX.repeatCount = ValueAnimator.INFINITE
        scaleY.repeatCount = ValueAnimator.INFINITE
        alpha.repeatCount = ValueAnimator.INFINITE

        pulseAnimator = AnimatorSet().apply {
            playTogether(scaleX, scaleY, alpha)
            duration = 2000
            interpolator = AccelerateDecelerateInterpolator()
            start()
        }
    }

    private fun stopPulseAnimation() {
        pulseAnimator?.cancel()
        pulseBg.alpha = 0f
        pulseBg.visibility = View.GONE
    }
}
