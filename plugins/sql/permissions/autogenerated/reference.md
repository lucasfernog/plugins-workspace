## Default Permission

### Default Permissions

This permission set configures what kind of
database operations are available from the sql plugin.

### Granted Permissions

Allows to load or close a connection and to run queries
with `select`. Running statements with `execute` is not
enabled by default.

Note that this is not a read-only mode: `select` runs any
SQL statement it is given, including ones that modify the
database.

#### This default permission set includes the following:

- `allow-close`
- `allow-load`
- `allow-select`

## Permission Table

<table>
<tr>
<th>Identifier</th>
<th>Description</th>
</tr>


<tr>
<td>

`sql:allow-close`

</td>
<td>

Enables the close command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`sql:deny-close`

</td>
<td>

Denies the close command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`sql:allow-execute`

</td>
<td>

Enables the execute command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`sql:deny-execute`

</td>
<td>

Denies the execute command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`sql:allow-load`

</td>
<td>

Enables the load command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`sql:deny-load`

</td>
<td>

Denies the load command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`sql:allow-select`

</td>
<td>

Enables the select command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`sql:deny-select`

</td>
<td>

Denies the select command without any pre-configured scope.

</td>
</tr>
</table>
