pub(crate) const DEFAULT_CONFIG: &str = "
theme = 'dark'
max_results = 9
abs_paths = true
compact_paths = true
database_path = '$HOME/.local/share/cz/'
";


pub(crate) const HELP: &str = "
Usage: z [OPTION]... [DIRECTORY]...

Mandatory arguments to long options are mandatory for short options too.
  -                          go to the previous directory
  =                          go to the current directory
  -a                         add directory alias; if only the directory is
                               introduced, its alias is removed
  -b                         execute shell command in background
      --clear                clear the directories database
  -i                         interactive selection (using a numbered list) of
                               the subdirectories of the current directory.
  -l[=NUMBER]                list a certain NUMBER of directories by 'frecency';
                               if no NUMBER is provided, the max_results number
                               from configuration is used.
  -r=DIRECTORY               remove DIRECTORY from the database
      --help     display this help and exit

Exit status:
 0  if OK,
 1  if minor problems (e.g., cannot access subdirectory)

Full documentation <https://github.com/sonarom/contemporary-z>
";
