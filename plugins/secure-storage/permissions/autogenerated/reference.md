## Default Permission

This permission set configures what kind of
operations are available from the secure storage plugin.

#### Granted Permissions

Reading, writing and removing items is enabled by default.
Items are scoped to the application, so this does not
expose secrets stored by other applications.

#### This default permission set includes the following:

- `allow-get-item`
- `allow-set-item`
- `allow-remove-item`

## Permission Table

<table>
<tr>
<th>Identifier</th>
<th>Description</th>
</tr>


<tr>
<td>

`secure-storage:allow-get-item`

</td>
<td>

Enables the get_item command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`secure-storage:deny-get-item`

</td>
<td>

Denies the get_item command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`secure-storage:allow-remove-item`

</td>
<td>

Enables the remove_item command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`secure-storage:deny-remove-item`

</td>
<td>

Denies the remove_item command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`secure-storage:allow-set-item`

</td>
<td>

Enables the set_item command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`secure-storage:deny-set-item`

</td>
<td>

Denies the set_item command without any pre-configured scope.

</td>
</tr>
</table>
