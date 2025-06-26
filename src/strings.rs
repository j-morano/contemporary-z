pub(crate) const DEFAULT_CONFIG: &str = "\
theme = dark
max_results = 9
abs_paths = true
compact_paths = true
database_path = $HOME/.local/share/contemporary-z/directories.dir
substring = score
show_files = none
nav_start_number = 1
";


pub(crate) const HELP: &str = "\
Usage: z [OPTION]... [DIRECTORY|SUBSTRING]...

FUNCTIONALITY:
  * If no option nor directory or substrings are specified, 'cz' prints a
    numbered list of the most 'frecent' directories to select one of them by
    introducing its number.
  * If a directory alias is introduced, 'cz' does 'cd' to the directory.
  * If a directory name is introduced, 'cz' jumps to the directory (if
    available) and adds it to the directories database (if it is not already
    added).
  * If a substring or substrings are introduced, 'cz' searches in the database
    for coincidences. The selected directory will depend on the 'substring'
    configuration option (see Configuration section below). If there is only
    one result, it always 'cd's to it, regardless of the option.

OPTIONS:
Mandatory arguments to long options are mandatory for short options too.
  -                          go to the previous directory.
  =                          go to the current directory.
  -a [ALIAS] DIRECTORY       add directory alias; if only the directory is
                               introduced, its alias is removed; if only an
                               alias is introduced, the programs shows the
                               directory list to select one of them.
      --clear                clear the directories database.
  -b SUBSTRING               force substring matching by basename.
  --database-path            show the path of the directories database.
  -e SUBSTRING               force substring matching by score.
  -f SUBSTRING               force substring match list for SUBSTRING
  -i                         interactive selection (using a numbered list) of
                               the subdirectories of the current directory.
      --ih                   interactive selection, but including hidden
                               directories.
      --id                   interactive selection, for directories only
                               (ignore congifuration option 'show_files').
  -l [NUMBER]                list a certain NUMBER of directories by 'frecency';
                               if no NUMBER is provided, the max_results number
                               from configuration is used.
      --list-all             list all the directories of the database
  -r                         remove a directory from the database, interactively.
      --remove-alias         remove an alias, interactively.
  -t SUBSTRING               force substring matching by shortest path.
      --sync                 sync directories (remove all non-existent
                                directories).
      --help     display this help and exit.
  -v, --version              display version information and exit.

Exit status:
 0  if OK,
 1  if minor problems (e.g., cannot access subdirectory)


CONFIGURATION:
'cz' supports some configuration options. These options must be set in a file
with the following path: '~/.config/contemporary-z/cz.conf' using the format
'option = value'.

Configuration options:
  theme: string. Color theme.
      * Allowed values: 'dark', 'bright'
  abs_paths: bool. Record directories using full paths or relative paths.
      With the latter option, shown directories will vary from one directory to
      another.
  compact_paths: bool. Replace '/home/<username>' by '~' and
      '/run/media/<username>' by '>'.
  max_results: int. Maximum results to show in the directory list.
  database_path: string. Directory where the directories database is/will
      be located.
  substring: string. Which dir to select when substring(s) are introduced.
      * Allowed values:
          - 'shortest': go to the directory with the shortest path name.
          - 'score': go to the directory with the highest score (most 'frecent'
              dir).
          - 'none': show selection list.
          - 'basename': go to the directory with the highest score (most 'frecent'
              dir) whose _basename_ matches the substring(s).
  show_files: string. Whether to show non-dir files, and where, in
      interactive selection.
      * Allowed values:
          - 'top': show files on top of dirs.
          - 'bottom': show files under the dirs.
          - 'none': do not show files.
  nav_start_number: int. Start number for interactive navigation, that is,
      the number that the parent directory will have.
      * Recommended values: 1 or 0.


Default config:
-------------------------------------------------------------------
# ~/.config/contemporary-z/cz.conf

theme = dark
max_results = 9
abs_paths = true
compact_paths = true
database_path = $HOME/.local/share/contemporary-z/directories.dir
substring = score
show_files = none
nav_start_number = 1
-------------------------------------------------------------------

Source code: <https://github.com/j-morano/contemporary-z>
";
