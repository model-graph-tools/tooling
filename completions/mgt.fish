# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_mgt_global_optspecs
	string join \n h/help V/version
end

function __fish_mgt_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_mgt_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_mgt_using_subcommand
	set -l cmd (__fish_mgt_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c mgt -n "__fish_mgt_needs_command" -s h -l help -d 'Print help'
complete -c mgt -n "__fish_mgt_needs_command" -s V -l version -d 'Print version'
complete -c mgt -n "__fish_mgt_needs_command" -f -a "analyze" -d 'Analyze the management model of a WildFly instance and build an image with a Neo4J database'
complete -c mgt -n "__fish_mgt_needs_command" -f -a "neo4j" -d 'Start and stop a Neo4J model database'
complete -c mgt -n "__fish_mgt_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c mgt -n "__fish_mgt_using_subcommand analyze" -s h -l help -d 'Print help'
complete -c mgt -n "__fish_mgt_using_subcommand analyze" -s V -l version -d 'Print version'
complete -c mgt -n "__fish_mgt_using_subcommand neo4j; and not __fish_seen_subcommand_from start help" -s h -l help -d 'Print help'
complete -c mgt -n "__fish_mgt_using_subcommand neo4j; and not __fish_seen_subcommand_from start help" -s V -l version -d 'Print version'
complete -c mgt -n "__fish_mgt_using_subcommand neo4j; and not __fish_seen_subcommand_from start help" -f -a "start" -d 'Start one or several Neo4J model databases'
complete -c mgt -n "__fish_mgt_using_subcommand neo4j; and not __fish_seen_subcommand_from start help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c mgt -n "__fish_mgt_using_subcommand neo4j; and __fish_seen_subcommand_from start" -s h -l help -d 'Print help'
complete -c mgt -n "__fish_mgt_using_subcommand neo4j; and __fish_seen_subcommand_from start" -s V -l version -d 'Print version'
complete -c mgt -n "__fish_mgt_using_subcommand neo4j; and __fish_seen_subcommand_from start" -a "stop" -d 'Stop one or several Neo4J model databases'
complete -c mgt -n "__fish_mgt_using_subcommand neo4j; and __fish_seen_subcommand_from start" -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c mgt -n "__fish_mgt_using_subcommand neo4j; and __fish_seen_subcommand_from help" -f -a "start" -d 'Start one or several Neo4J model databases'
complete -c mgt -n "__fish_mgt_using_subcommand neo4j; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c mgt -n "__fish_mgt_using_subcommand help; and not __fish_seen_subcommand_from analyze neo4j help" -f -a "analyze" -d 'Analyze the management model of a WildFly instance and build an image with a Neo4J database'
complete -c mgt -n "__fish_mgt_using_subcommand help; and not __fish_seen_subcommand_from analyze neo4j help" -f -a "neo4j" -d 'Start and stop a Neo4J model database'
complete -c mgt -n "__fish_mgt_using_subcommand help; and not __fish_seen_subcommand_from analyze neo4j help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c mgt -n "__fish_mgt_using_subcommand help; and __fish_seen_subcommand_from neo4j" -f -a "start" -d 'Start one or several Neo4J model databases'
