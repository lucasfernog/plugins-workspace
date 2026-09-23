---
"haptics": patch
"haptics-js": patch
---

Fixed the `Error` type of the JavaScript API: it was `never`, but a failed command resolves to `{ status: 'error', error }` with the error message as a `string`. The documentation now states that the functions resolve with an error result instead of rejecting.
