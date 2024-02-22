### Description

This tool fetches the latest hashes from providers and builds its own files for use with Raspirus (https://github.com/Raspirus/raspirus.git).

### Functionality

All the arguments can be combined with eachother. The program goes through them one by one, so the user can declare the order in which operations are executed.

The provided hashes on our repo are generated using this command, ran in signature_builder with signatures repo cloned in the same folder: `cargo run --release -- -mc 64 -ct -cdb -u -p ../signatures/patches/patch_1 -dd -o ../signatures/hashes/ -e -s`

<p> </p>

`-h`|`--help`

Prints the help prompt. This is probably where you want to start if you are struggling

<p> </p>

`-f`|`--fetch`

Fetches the files from all providers and saves them to the temporary working directory

<p> </p>

`-i`|`--insert`

Tries to insert all files from the temporary working directory into the database

<p> </p>

`-e`|`--export`

Exports all hashes into the output directory. Splits into multiple files with the maximum line number

<p> </p>

`-if`|`--insert-file` [`filename`]

Tries to insert the file provided into the Database

<p> </p>

`-u`|`--update`

Fetches the latest files form all providers, saves them to the temporary working directory and tries to import them into the database. Basically the same as running the tool with `-f -i`

<p> </p>

`-cdb`|`--clean-database`

Removes the database. !USE WITH CAUTION!

<p> </p>

`-ct`|`--clean-temp`

Removes the temporary folder. !USE WITH CAUTION!

<p> </p>

`-cd`|`--clean-data`

Removes the table in the database. !USE WITH CAUTION!

<p> </p>

`-p`|`--patch` [`filename`]

Patches the database with a patch file. The file should contain one hash on each line, prefixed with `+` or `-`, depending on if it should be added or removed from the database.

Example:

```
# Any line prefixed with anything other than + or - will be ignored
+ 2d75cc1bf8e57872781f9cd04a529256
- 7dea362b3fac8e00956a4952a3d4f474
```

In this example, the line with the # will be ignored while also outputting a warning to notify the user of the skipped line. The space between the prefix and hash are optional

<p> </p>

`-n`|`--numerate`

Returns the number of hashes currently in the database

<p> </p>

`-s`|`--set-time`

Sets the timestamp of the output folder

<p> </p>

`-dd`|`--de-dup`

Removes duplicates from table

<p> </p>

`-t`|`--tempdir` [`foldername`]

Sets the temporary working directory that will be created to `foldername`. Use with caution as it will modify preexisting folders. Useful if you wish to keep multiple working directories or delete a specific one using `-c`. Defaults to `./tmp`

<p> </p>

`-d`|`--database` [`database`]

Sets the databas name to `database`. Useful if you wish to keep multiple databases or delete a specific one using `-c`. Defaults to `hashes_db`

<p> </p>

`-mt`|`--max-threads` [`threadcount`]

Sets the maximum number of parallel download threads to `threadcount`. Numbers too high will result in timouts. Defaults to `20`

<p> </p>

`-mr`|`--max-retries` [`retrycount`]

Sets the maximum number of retires for failed downloads to `retrycount`. Defaults to `5`

<p> </p>

`-mc`|`--max-combines` [`filecount`]

Sets how many files can be combined for inserting to `filecount`. Can be used to speed up insertion at the cost of memory. Defaults to `8`, since this will pretty much run on anything

<p> </p>

`-tb`|`--table` [`tablename`]

Sets the database table to `tablename`. Can be used if you wish to keep a single database for multiple runs.  Defaults to `hashes`

<p> </p>

`-o`|`--output` [`foldername`]

Sets the output folder to `foldername`. Useful if you wish to output the created hashfiles to a separate folder like an external git repo. Defaults to `./hashes`

<p> </p>

`-l`|`--length` [`length`]

Sets the number of lines contained in each of the output files to `length`. Useful if you wish to create fewer, larger files for easy storage or smaller ones if you face file size limits. Defaults to `1_000_000`.

Note: Numbers cannot be entered as `1_000`, but have to entered as `1000`. The previous notation is just for readability

### Logging

The default verbosity of the tool (INFO) can be changed by setting the environment variable SB_LOG to `INFO`, `DEBUG`, `TRACE` or `ERROR`.
