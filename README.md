### Functionality

`-h`|`--help`

Prints the help prompt. This is probably where you want to start if you are struggling

<p>&nbsp;</p>

`-f`|`--fetch`

Fetches the files from all providers and saves them to the temporary working directory

<p>&nbsp;</p>

`-i`|`--insert`

Tries to insert all files from the temporary working directory into the database

<p>&nbsp;</p>

`-e`|`--export`

Exports all hashes into the output directory. Splits into multiple files with the maximum line number

<p>&nbsp;</p>

`-if`|`--insert-file` [`filename`]

Tries to insert the file provided into the Database

<p>&nbsp;</p>

`-u`|`--update`

Fetches the latest files form all providers, saves them to the temporary working directory and tries to import them into the database. Basically the same as running the tool with `-f -i`

<p>&nbsp;</p>

`-c`|`--clean`

Removes the temporary working directory and the database. !USE WITH CAUTION!

<p>&nbsp;</p>

`-p`|`--patch` [`filename`]

Patches the database with a patch file. The file should contain one hash on each line, prefixed with `+` or `-`, depending on if it should be added or removed from the database.

Example:

```
# Any line prefixed with anything other than + or - will be ignored
+ 2d75cc1bf8e57872781f9cd04a529256
- 00f538c3d410822e241486ca061a57ee
```

In this example, the line with the # will be ignored while also outputting a warning to notify the user of the skipped line. The space between the prefix and hash are optional

<p>&nbsp;</p>

`-n`|`--numerate`

Returns the number of hashes currently in the database

<p>&nbsp;</p>

`-t`|`--tempdir` [`foldername`]

Sets the temporary working directory that will be created to `foldername`. Use with caution as it will modify preexisting folders. Useful if you wish to keep multiple working directories or delete a specific one using `-c`. Defaults to `./tmp`

<p>&nbsp;</p>

`-d`|`--database` [`database`]

Sets the databas name to `database`. Useful if you wish to keep multiple databases or delete a specific one using `-c`. Defaults to `hashes_db`

<p>&nbsp;</p>

`-mt`|`--max-threads` [`threadcount`]

Sets the maximum number of parallel download threads to `threadcount`. Numbers too high will result in timouts. Defaults to `20`

<p>&nbsp;</p>

`-mr`|`--max-retries` [`retrycount`]

Sets the maximum number of retires for failed downloads to `retrycount`. Defaults to `5`

<p>&nbsp;</p>

`-mc`|`--max-combines` [`filecount`]

Sets how many files can be combined for inserting to `filecount`. Can be used to speed up insertion at the cost of memory. Defaults to `8`, since this will pretty much run on anything

<p>&nbsp;</p>

`-tb`|`--table` [`tablename`]

Sets the database table to `tablename`. Can be used if you wish to keep a single database for multiple runs.  Defaults to `hashes`

<p>&nbsp;</p>

`-o`|`--output` [`foldername`]

Sets the output folder to `foldername`. Useful if you wish to output the created hashfiles to a separate folder like an external git repo. Defaults to `./hashes`

<p>&nbsp;</p>

`-l`|`--length` [`length`]

Sets the number of lines contained in each of the output files to `length`. Useful if you wish to create fewer, larger files for easy storage or smaller ones if you face file size limits. Defaults to `1_000_000`.

Note: Numbers cannot be entered as `1_000`, but have to entered as `1000`. The previous notation is just for readability

### Logging

The default verbosity of the tool (INFO) can be changed by setting the environment variable SB_LOG to `INFO`, `DEBUG`, `TRACE` or `ERROR`.
