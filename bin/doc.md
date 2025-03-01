Search options
   -r <search>, -regex <search>
        Search using regular expressions.
   -i, -case
        Match case.
   -w, -ww, -whole-word, -whole-words
        Match whole words.
   -p, -match-path
        Match full path and file name.
   -a, -diacritics
        Match diacritical marks.

   -o <offset>, -offset <offset>
        Show results starting from offset.
   -n <num>, -max-results <num>
        Limit the number of results shown to <num>.

   -path <path>
        Search for subfolders and files in path.
   -parent-path <path>
        Search for subfolders and files in the parent of path.
   -parent <path>
        Search for files with the specified parent path.

   /ad
        Folders only.
   /a-d
        Files only.
   /a[RHSDAVNTPLCOIE]
        DIR style attributes search.
        R = Read only.
        H = Hidden.
        S = System.
        D = Directory.
        A = Archive.
        V = Device.
        N = Normal.
        T = Temporary.
        P = Sparse file.
        L = Reparse point.
        C = Compressed.
        O = Offline.
        I = Not content indexed.
        E = Encrypted.
        - = Prefix a flag with - to exclude.


Sort options
   -s
        sort by full path.
   -sort <name[-ascending|-descending]>, -sort-<name>[-ascending|-descending]
        Set sort
        name=name|path|size|extension|date-created|date-modified|date-accessed|
        attributes|file-list-file-name|run-count|date-recently-changed|date-run
   -sort-ascending, -sort-descending
        Set sort order

   /on, /o-n, /os, /o-s, /oe, /o-e, /od, /o-d
        DIR style sorts.
        N = Name.
        S = Size.
        E = Extension.
        D = Date modified.
        - = Sort in descending order.


Display options
   -name
   -path-column
   -full-path-and-name, -filename-column
   -extension, -ext
   -size
   -date-created, -dc
   -date-modified, -dm
   -date-accessed, -da
   -attributes, -attribs, -attrib
   -file-list-file-name
   -run-count
   -date-run
   -date-recently-changed, -rc
        Show the specified column.

   -highlight
        Highlight results.
   -highlight-color <color>
        Highlight color 0-255.

   -csv
   -efu
   -txt
   -m3u
   -m3u8
   -tsv
        Change display format.

   -size-format <format>
        0=auto, 1=Bytes, 2=KB, 3=MB.
   -date-format <format>
        0=auto, 1=ISO-8601, 2=FILETIME, 3=ISO-8601(UTC)

   -filename-color <color>
   -name-color <color>
   -path-color <color>
   -extension-color <color>
   -size-color <color>
   -date-created-color <color>, -dc-color <color>
   -date-modified-color <color>, -dm-color <color>
   -date-accessed-color <color>, -da-color <color>
   -attributes-color <color>
   -file-list-filename-color <color>
   -run-count-color <color>
   -date-run-color <color>
   -date-recently-changed-color <color>, -rc-color <color>
        Set the column color 0-255.

   -filename-width <width>
   -name-width <width>
   -path-width <width>
   -extension-width <width>
   -size-width <width>
   -date-created-width <width>, -dc-width <width>
   -date-modified-width <width>, -dm-width <width>
   -date-accessed-width <width>, -da-width <width>
   -attributes-width <width>
   -file-list-filename-width <width>
   -run-count-width <width>
   -date-run-width <width>
   -date-recently-changed-width <width>, -rc-width <width>
        Set the column width 0-200.

   -no-digit-grouping
        Don't group numbers with commas.
   -size-leading-zero
   -run-count-leading-zero
        Format the number with leading zeros, use with -no-digit-grouping.
   -double-quote
        Wrap paths and filenames with double quotes.


Export options
   -export-csv <out.csv>
   -export-efu <out.efu>
   -export-txt <out.txt>
   -export-m3u <out.m3u>
   -export-m3u8 <out.m3u8>
   -export-tsv <out.txt>
        Export to a file using the specified layout.
   -no-header
        Do not output a column header for CSV, EFU and TSV files.
   -utf8-bom
        Store a UTF-8 byte order mark at the start of the exported file.


General options
   -h, -help
        Display this help.

   -instance <name>
        Connect to the unique Everything instance name.
   -ipc1, -ipc2
        Use IPC version 1 or 2.
   -pause, -more
        Pause after each page of output.
   -hide-empty-search-results
        Don't show any results when there is no search.
   -empty-search-help
        Show help when no search is specified.
   -timeout <milliseconds>
        Timeout after the specified number of milliseconds to wait for
        the Everything database to load before sending a query.

   -set-run-count <filename> <count>
        Set the run count for the specified filename.
   -inc-run-count <filename>
        Increment the run count for the specified filename by one.
   -get-run-count <filename>
        Display the run count for the specified filename.
   -get-result-count
        Display the result count for the specified search.
   -get-total-size
        Display the total result size for the specified search.
   -save-settings, -clear-settings
        Save or clear settings.
   -version
        Display ES major.minor.revision.build version and exit.
   -get-everything-version
        Display Everything major.minor.revision.build version and exit.
   -exit
        Exit Everything.
        Returns after Everything process closes.
   -save-db
        Save the Everything database to disk.
        Returns after saving completes.
   -reindex
        Force Everything to reindex.
        Returns after indexing completes.
   -no-result-error
        Set the error level if no results are found.


Notes
    Internal -'s in options can be omitted, eg: -nodigitgrouping
    Switches can also start with a /
    Use double quotes to escape spaces and switches.
    Switches can be disabled by prefixing them with no-, eg: -no-size.
    Use a ^ prefix or wrap with double quotes (") to escape \ & | > < ^
