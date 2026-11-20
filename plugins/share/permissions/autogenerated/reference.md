## Default Permission

This permission set configures which
share features are by default exposed.

#### Granted Permissions

It allows opening the system share sheet for text and URLs.
Sharing files additionally requires scope entries on the
`allow-share` permission, there is no default file scope.

#### This default permission set includes the following:

- `allow-share`

## Permission Table

<table>
<tr>
<th>Identifier</th>
<th>Description</th>
</tr>


<tr>
<td>

`share:allow-share`

</td>
<td>

Enables the share command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`share:deny-share`

</td>
<td>

Denies the share command without any pre-configured scope.

</td>
</tr>
</table>
