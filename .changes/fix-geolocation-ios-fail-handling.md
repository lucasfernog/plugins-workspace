---
geolocation: patch
geolocation-js: patch
---

On iOS, a location failure no longer rejects pending `requestPermissions` calls, and watchers no longer receive an error for the transient "location unknown" failure, which the system retries on its own.
