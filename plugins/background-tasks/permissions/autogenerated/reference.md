## Default Permission

Allows listing the scheduled background tasks.

Scheduling and cancelling tasks from the webview is not allowed by default. Grant
`allow-schedule`, `allow-cancel` and `allow-cancel-all` to the windows that need them.

#### This default permission set includes the following:

- `allow-get-scheduled`

## Permission Table

<table>
<tr>
<th>Identifier</th>
<th>Description</th>
</tr>


<tr>
<td>

`background-tasks:allow-cancel`

</td>
<td>

Enables the cancel command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`background-tasks:deny-cancel`

</td>
<td>

Denies the cancel command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`background-tasks:allow-cancel-all`

</td>
<td>

Enables the cancel_all command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`background-tasks:deny-cancel-all`

</td>
<td>

Denies the cancel_all command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`background-tasks:allow-get-scheduled`

</td>
<td>

Enables the get_scheduled command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`background-tasks:deny-get-scheduled`

</td>
<td>

Denies the get_scheduled command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`background-tasks:allow-schedule`

</td>
<td>

Enables the schedule command without any pre-configured scope.

</td>
</tr>

<tr>
<td>

`background-tasks:deny-schedule`

</td>
<td>

Denies the schedule command without any pre-configured scope.

</td>
</tr>
</table>
