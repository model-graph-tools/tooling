
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
            cand neo4j 'Start and stop a Neo4J model database'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'mgt;analyze'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'mgt;neo4j'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand start 'Start one or several Neo4J model databases'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'mgt;neo4j;start'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand stop 'Stop one or several Neo4J model databases'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'mgt;neo4j;start;stop'= {
            cand -a 'Stop all running Neo4J model databases.'
            cand --all 'Stop all running Neo4J model databases.'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'mgt;neo4j;start;help'= {
            cand stop 'Stop one or several Neo4J model databases'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'mgt;neo4j;start;help;stop'= {
        }
        &'mgt;neo4j;start;help;help'= {
        }
        &'mgt;neo4j;help'= {
            cand start 'Start one or several Neo4J model databases'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'mgt;neo4j;help;start'= {
            cand stop 'Stop one or several Neo4J model databases'
        }
        &'mgt;neo4j;help;start;stop'= {
        }
        &'mgt;neo4j;help;help'= {
        }
        &'mgt;help'= {
            cand analyze 'Analyze the management model of a WildFly instance and build an image with a Neo4J database'
            cand neo4j 'Start and stop a Neo4J model database'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'mgt;help;analyze'= {
        }
        &'mgt;help;neo4j'= {
            cand start 'Start one or several Neo4J model databases'
        }
        &'mgt;help;neo4j;start'= {
            cand stop 'Stop one or several Neo4J model databases'
        }
        &'mgt;help;neo4j;start;stop'= {
        }
        &'mgt;help;help'= {
        }
    ]
    $completions[$command]
}
