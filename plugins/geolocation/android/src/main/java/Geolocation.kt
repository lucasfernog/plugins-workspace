// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.geolocation

import android.annotation.SuppressLint
import android.content.Context
import android.location.Location
import android.location.LocationManager
import android.os.SystemClock
import androidx.core.location.LocationManagerCompat
import app.tauri.Logger
import com.google.android.gms.common.ConnectionResult
import com.google.android.gms.common.GoogleApiAvailability
import com.google.android.gms.location.FusedLocationProviderClient
import com.google.android.gms.location.LocationCallback
import com.google.android.gms.location.LocationRequest
import com.google.android.gms.location.LocationResult
import com.google.android.gms.location.LocationServices
import com.google.android.gms.location.Priority


private const val PLAY_SERVICES_UNAVAILABLE = "Google Play Services unavailable."

public class Geolocation(private val context: Context) {
    private var fusedLocationClient: FusedLocationProviderClient? = null
    // One callback per watcher (keyed by its channel id), so several watchers can run at once.
    private val locationCallbacks = HashMap<Long, LocationCallback>()

    fun isLocationServicesEnabled(): Boolean {
        val lm = context.getSystemService(Context.LOCATION_SERVICE) as LocationManager
        return LocationManagerCompat.isLocationEnabled(lm)
    }

    private fun isPlayServicesAvailable(): Boolean =
        GoogleApiAvailability.getInstance().isGooglePlayServicesAvailable(context) == ConnectionResult.SUCCESS

    /** The Fused provider priority for a request; the low-accuracy one depends on the network provider. */
    private fun locationPriority(enableHighAccuracy: Boolean): Int {
        if (enableHighAccuracy) {
            return Priority.PRIORITY_HIGH_ACCURACY
        }

        val lm = context.getSystemService(Context.LOCATION_SERVICE) as LocationManager
        var networkEnabled = false
        try {
            networkEnabled = lm.isProviderEnabled(LocationManager.NETWORK_PROVIDER)
        } catch (_: Exception) {
            Logger.error("isProviderEnabled failed")
        }

        return if (networkEnabled) Priority.PRIORITY_BALANCED_POWER_ACCURACY else Priority.PRIORITY_LOW_POWER
    }

    @SuppressLint("MissingPermission")
    fun sendLocation(enableHighAccuracy: Boolean, successCallback: (location: Location) -> Unit, errorCallback: (error: String) -> Unit) {
        if (!isPlayServicesAvailable()) {
            errorCallback(PLAY_SERVICES_UNAVAILABLE)
            return
        }
        if (!isLocationServicesEnabled()) {
            errorCallback("Location disabled.")
            return
        }

        LocationServices
            .getFusedLocationProviderClient(context)
            .getCurrentLocation(locationPriority(enableHighAccuracy), null)
            .addOnFailureListener { e -> e.message?.let { errorCallback(it) } }
            .addOnSuccessListener { location ->
                if (location == null) {
                    errorCallback("Location unavailable.")
                } else {
                    successCallback(location)
                }
            }
    }

    @Synchronized
    @SuppressLint("MissingPermission")
    fun requestLocationUpdates(id: Long, enableHighAccuracy: Boolean, timeout: Long, successCallback: (location: Location) -> Unit, errorCallback: (error: String) -> Unit) {
        if (!isPlayServicesAvailable()) {
            errorCallback(PLAY_SERVICES_UNAVAILABLE)
            return
        }

        clearLocationUpdates(id)
        val client = fusedLocationClient ?: LocationServices.getFusedLocationProviderClient(context)
        fusedLocationClient = client

        if (!isLocationServicesEnabled()) {
            errorCallback("Location disabled.")
            return
        }

        val locationRequest = LocationRequest.Builder(timeout)
            .setMaxUpdateDelayMillis(timeout)
            .setMinUpdateIntervalMillis(timeout)
            .setPriority(locationPriority(enableHighAccuracy))
            .build()

        val locationCallback =
            object : LocationCallback() {
                override fun onLocationResult(locationResult: LocationResult) {
                    val lastLocation = locationResult.lastLocation
                    if (lastLocation == null) {
                        errorCallback("Location unavailable.")
                    } else {
                        successCallback(lastLocation)
                    }
                }
            }

        client.requestLocationUpdates(locationRequest, locationCallback, null)
        locationCallbacks[id] = locationCallback
    }

    /** Stops the location updates of the watcher with the given id. */
    @Synchronized
    fun clearLocationUpdates(id: Long) {
        val callback = locationCallbacks.remove(id) ?: return
        fusedLocationClient?.removeLocationUpdates(callback)
    }

    /** Stops the location updates of every watcher. */
    @Synchronized
    fun clearAllLocationUpdates() {
        for (callback in locationCallbacks.values) {
            fusedLocationClient?.removeLocationUpdates(callback)
        }
        locationCallbacks.clear()
    }

    @SuppressLint("MissingPermission")
    fun getLastLocation(maximumAge: Long): Location? {
        var lastLoc: Location? = null
        val lm = context.getSystemService(Context.LOCATION_SERVICE) as LocationManager

        for (provider in lm.allProviders) {
            val tmpLoc = lm.getLastKnownLocation(provider)
            if (tmpLoc != null) {
                val locationAge = SystemClock.elapsedRealtimeNanos() - tmpLoc.elapsedRealtimeNanos
                val maxAgeNano = maximumAge * 1000000L
                if (locationAge <= maxAgeNano && (lastLoc == null || lastLoc.elapsedRealtimeNanos > tmpLoc.elapsedRealtimeNanos)) {
                    lastLoc = tmpLoc
                }
            }
        }

        return lastLoc
    }
}
