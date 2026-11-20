# The native methods of TaskRunner are looked up by name from Rust (JNI) and
# WorkManager instantiates the worker by its class name.
-keep class app.tauri.backgroundtasks.TaskRunner { *; }
-keep class app.tauri.backgroundtasks.BackgroundTaskWorker { *; }
