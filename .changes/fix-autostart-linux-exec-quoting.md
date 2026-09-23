---
autostart: patch
autostart-js: patch
---

Fixed autostart on Linux when the executable or AppImage path, or a startup argument, contains spaces or other reserved characters. They are now quoted in the desktop entry's `Exec` key as required by the Desktop Entry specification.
