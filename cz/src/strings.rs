pub(crate) const DEFAULT_CONFIG: &str = "\
theme = 'dark'
max_results = 9
abs_paths = true
compact_paths = true
database_path = '$HOME/.local/share/contemporary-z/directories.db'
substring = 'shortest'
show_files = 'none'
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
    for coincidences. Then, if 'substring' is 'shortest' or the number of
    matches is equal to 1, it 'cd's to the directory with the shortest pathname.
    If 'substring' is equal to 'score', it goes to the directory with the
    highest score. Otherwise, it prints the interactive selection menu. If there
    is only one result, it always 'cd's to it.

OPTIONS:
Mandatory arguments to long options are mandatory for short options too.
  -                          go to the previous directory.
  =                          go to the current directory.
  -a [ALIAS] DIRECTORY       add directory alias; if only the directory is
                               introduced, its alias is removed; if only an
                               alias is introduced, the programs shows the
                               directory list to select one of them.
      --clear                clear the directories database.
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
      --sync                 sync directories (remove all non-existent
                                directories).
      --help     display this help and exit.
  -v, --version              display version information and exit.

Exit status:
 0  if OK,
 1  if minor problems (e.g., cannot access subdirectory)

Full documentation <https://github.com/sonarom/contemporary-z>
";
