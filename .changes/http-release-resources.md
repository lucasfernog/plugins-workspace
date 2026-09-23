---
http: patch
http-js: patch
---

Fixed resource leaks in the webview's resource table: a request and its abort handle are now released once `fetch_send` completes (successfully, with an error or aborted) or when a request is aborted before it is sent, and a response body is released right away for null-body statuses (such as `204`) and when its stream is garbage collected without being read to the end.
