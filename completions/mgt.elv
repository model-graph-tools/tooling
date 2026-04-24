
use builtin;
use str;

set edit:completion:arg-completer[mgt] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'mgt'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'mgt'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand analyze 'Analyze the management model of a WildFly instance and build an image with a Neo4J database'
            cand start 'Start Neo4J model DB containers'
            cand stop 'Stop Neo4J model DB containers'
            cand browse 'Open the Neo4J browser for a running Neo4J model DB'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'mgt;analyze'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'mgt;start'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'mgt;stop'= {
            cand -a 'Stop all running Neo4J model DB containers'
            cand --all 'Stop all running Neo4J model DB containers'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'mgt;browse'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'mgt;help'= {
            cand analyze 'Analyze the management model of a WildFly instance and build an image with a Neo4J database'
            cand start 'Start Neo4J model DB containers'
            cand stop 'Stop Neo4J model DB containers'
            cand browse 'Open the Neo4J browser for a running Neo4J model DB'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'mgt;help;analyze'= {
        }
        &'mgt;help;start'= {
        }
        &'mgt;help;stop'= {
        }
        &'mgt;help;browse'= {
        }
        &'mgt;help;help'= {
        }
    ]
    $completions[$command]
}
