# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_wdc_global_optspecs
	string join \n h/help V/version
end

function __fish_wdc_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_wdc_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_wdc_using_subcommand
	set -l cmd (__fish_wdc_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c wdc -n "__fish_wdc_needs_command" -s h -l help -d 'Print help'
complete -c wdc -n "__fish_wdc_needs_command" -s V -l version -d 'Print version'
complete -c wdc -n "__fish_wdc_needs_command" -f -a "build" -d 'Build WildFly images'
complete -c wdc -n "__fish_wdc_needs_command" -f -a "start" -d 'Start a standalone server'
complete -c wdc -n "__fish_wdc_needs_command" -f -a "stop" -d 'Stop a standalone server'
complete -c wdc -n "__fish_wdc_needs_command" -f -a "dc" -d 'Start and stop a domain controller'
complete -c wdc -n "__fish_wdc_needs_command" -f -a "hc" -d 'Start and stop a host controller'
complete -c wdc -n "__fish_wdc_needs_command" -f -a "topology" -d 'Start and stop a topology defined as YAML'
complete -c wdc -n "__fish_wdc_needs_command" -f -a "console" -d 'Open the management console'
complete -c wdc -n "__fish_wdc_needs_command" -f -a "cli" -d 'Connect to the CLI'
complete -c wdc -n "__fish_wdc_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wdc -n "__fish_wdc_using_subcommand build" -s u -l username -d 'The username of the management user' -r
complete -c wdc -n "__fish_wdc_using_subcommand build" -s s -l password -d 'The password of the management user' -r
complete -c wdc -n "__fish_wdc_using_subcommand build" -l chunks -d 'Build the images in chunks of this size. If not specified, the images are built in one go.' -r
complete -c wdc -n "__fish_wdc_using_subcommand build" -s l -l latest -d 'Mark the version as the latest version. If used with a version range, the latest tag is applied to the largest version in the range.'
complete -c wdc -n "__fish_wdc_using_subcommand build" -s h -l help -d 'Print help'
complete -c wdc -n "__fish_wdc_using_subcommand build" -s V -l version -d 'Print version'
complete -c wdc -n "__fish_wdc_using_subcommand start" -s n -l name -d 'The name of the standalone server' -r
complete -c wdc -n "__fish_wdc_using_subcommand start" -s p -l http -d 'The published HTTP port' -r
complete -c wdc -n "__fish_wdc_using_subcommand start" -s m -l management -d 'The published management port' -r
complete -c wdc -n "__fish_wdc_using_subcommand start" -s o -l offset -d 'The offset added to the published HTTP and management ports' -r
complete -c wdc -n "__fish_wdc_using_subcommand start" -s h -l help -d 'Print help'
complete -c wdc -n "__fish_wdc_using_subcommand start" -s V -l version -d 'Print version'
complete -c wdc -n "__fish_wdc_using_subcommand stop" -s n -l name -d 'The name of the standalone server' -r
complete -c wdc -n "__fish_wdc_using_subcommand stop" -s h -l help -d 'Print help'
complete -c wdc -n "__fish_wdc_using_subcommand stop" -s V -l version -d 'Print version'
complete -c wdc -n "__fish_wdc_using_subcommand dc; and not __fish_seen_subcommand_from start stop help" -s h -l help -d 'Print help'
complete -c wdc -n "__fish_wdc_using_subcommand dc; and not __fish_seen_subcommand_from start stop help" -s V -l version -d 'Print version'
complete -c wdc -n "__fish_wdc_using_subcommand dc; and not __fish_seen_subcommand_from start stop help" -f -a "start" -d 'Start a domain controller'
complete -c wdc -n "__fish_wdc_using_subcommand dc; and not __fish_seen_subcommand_from start stop help" -f -a "stop" -d 'Stop a domain controller'
complete -c wdc -n "__fish_wdc_using_subcommand dc; and not __fish_seen_subcommand_from start stop help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wdc -n "__fish_wdc_using_subcommand dc; and __fish_seen_subcommand_from start" -s n -l name -d 'The name of the domain controller' -r
complete -c wdc -n "__fish_wdc_using_subcommand dc; and __fish_seen_subcommand_from start" -s p -l http -d 'The published HTTP port' -r
complete -c wdc -n "__fish_wdc_using_subcommand dc; and __fish_seen_subcommand_from start" -s m -l management -d 'The published management port' -r
complete -c wdc -n "__fish_wdc_using_subcommand dc; and __fish_seen_subcommand_from start" -s o -l offset -d 'The offset added to the published HTTP and management ports' -r
complete -c wdc -n "__fish_wdc_using_subcommand dc; and __fish_seen_subcommand_from start" -s s -l server -d 'Manage servers of the domain controller. Servers are specified as <name>[:<server-group>][:<offset>][:start][:publish] <name>          The name of the server. This part is mandatory and must be first.                 All other parts are optional and can follow in any order. <server-group>  The name of the server group. Allowed values are \'main-server-group\' or \'msg\',                 and \'other-server-group\' or \'osg\'. If not specified, \'main-server-group\' is used. <offset>        The port offset. If not specified, 100 is used from the second server onwards. start           Whether to start the server. publish         Whether to publish the HTTP port.' -r
complete -c wdc -n "__fish_wdc_using_subcommand dc; and __fish_seen_subcommand_from start" -s h -l help -d 'Print help'
complete -c wdc -n "__fish_wdc_using_subcommand dc; and __fish_seen_subcommand_from start" -s V -l version -d 'Print version'
complete -c wdc -n "__fish_wdc_using_subcommand dc; and __fish_seen_subcommand_from stop" -s n -l name -d 'The name of the domain controller' -r
complete -c wdc -n "__fish_wdc_using_subcommand dc; and __fish_seen_subcommand_from stop" -s h -l help -d 'Print help'
complete -c wdc -n "__fish_wdc_using_subcommand dc; and __fish_seen_subcommand_from stop" -s V -l version -d 'Print version'
complete -c wdc -n "__fish_wdc_using_subcommand dc; and __fish_seen_subcommand_from help" -f -a "start" -d 'Start a domain controller'
complete -c wdc -n "__fish_wdc_using_subcommand dc; and __fish_seen_subcommand_from help" -f -a "stop" -d 'Stop a domain controller'
complete -c wdc -n "__fish_wdc_using_subcommand dc; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wdc -n "__fish_wdc_using_subcommand hc; and not __fish_seen_subcommand_from start stop help" -s h -l help -d 'Print help'
complete -c wdc -n "__fish_wdc_using_subcommand hc; and not __fish_seen_subcommand_from start stop help" -s V -l version -d 'Print version'
complete -c wdc -n "__fish_wdc_using_subcommand hc; and not __fish_seen_subcommand_from start stop help" -f -a "start" -d 'Start a host controller'
complete -c wdc -n "__fish_wdc_using_subcommand hc; and not __fish_seen_subcommand_from start stop help" -f -a "stop" -d 'Stop a host controller'
complete -c wdc -n "__fish_wdc_using_subcommand hc; and not __fish_seen_subcommand_from start stop help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wdc -n "__fish_wdc_using_subcommand hc; and __fish_seen_subcommand_from start" -s d -l domain-controller -d 'The name of the domain controller' -r
complete -c wdc -n "__fish_wdc_using_subcommand hc; and __fish_seen_subcommand_from start" -s n -l name -d 'The name of the host controller' -r
complete -c wdc -n "__fish_wdc_using_subcommand hc; and __fish_seen_subcommand_from start" -s s -l server -d 'Manage servers of the host controller. Servers are specified as <name>[:<server-group>][:<offset>][:start][:publish] <name>          The name of the server. This part is mandatory and must be first.                 All other parts are optional and can follow in any order. <server-group>  The name of the server group. Allowed values are \'main-server-group\' or \'msg\',                 and \'other-server-group\' or \'osg\'. If not specified, \'main-server-group\' is used. <offset>        The port offset. If not specified, 100 is used from the second server onwards. start           Whether to start the server. publish         Whether to publish the HTTP port.' -r
complete -c wdc -n "__fish_wdc_using_subcommand hc; and __fish_seen_subcommand_from start" -s h -l help -d 'Print help'
complete -c wdc -n "__fish_wdc_using_subcommand hc; and __fish_seen_subcommand_from start" -s V -l version -d 'Print version'
complete -c wdc -n "__fish_wdc_using_subcommand hc; and __fish_seen_subcommand_from stop" -s n -l name -d 'The name of the host controller' -r
complete -c wdc -n "__fish_wdc_using_subcommand hc; and __fish_seen_subcommand_from stop" -s h -l help -d 'Print help'
complete -c wdc -n "__fish_wdc_using_subcommand hc; and __fish_seen_subcommand_from stop" -s V -l version -d 'Print version'
complete -c wdc -n "__fish_wdc_using_subcommand hc; and __fish_seen_subcommand_from help" -f -a "start" -d 'Start a host controller'
complete -c wdc -n "__fish_wdc_using_subcommand hc; and __fish_seen_subcommand_from help" -f -a "stop" -d 'Stop a host controller'
complete -c wdc -n "__fish_wdc_using_subcommand hc; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wdc -n "__fish_wdc_using_subcommand topology; and not __fish_seen_subcommand_from start stop help" -s h -l help -d 'Print help'
complete -c wdc -n "__fish_wdc_using_subcommand topology; and not __fish_seen_subcommand_from start stop help" -s V -l version -d 'Print version'
complete -c wdc -n "__fish_wdc_using_subcommand topology; and not __fish_seen_subcommand_from start stop help" -f -a "start" -d 'Start a topology'
complete -c wdc -n "__fish_wdc_using_subcommand topology; and not __fish_seen_subcommand_from start stop help" -f -a "stop" -d 'Stop a topology'
complete -c wdc -n "__fish_wdc_using_subcommand topology; and not __fish_seen_subcommand_from start stop help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wdc -n "__fish_wdc_using_subcommand topology; and __fish_seen_subcommand_from start" -s h -l help -d 'Print help'
complete -c wdc -n "__fish_wdc_using_subcommand topology; and __fish_seen_subcommand_from start" -s V -l version -d 'Print version'
complete -c wdc -n "__fish_wdc_using_subcommand topology; and __fish_seen_subcommand_from stop" -s h -l help -d 'Print help'
complete -c wdc -n "__fish_wdc_using_subcommand topology; and __fish_seen_subcommand_from stop" -s V -l version -d 'Print version'
complete -c wdc -n "__fish_wdc_using_subcommand topology; and __fish_seen_subcommand_from help" -f -a "start" -d 'Start a topology'
complete -c wdc -n "__fish_wdc_using_subcommand topology; and __fish_seen_subcommand_from help" -f -a "stop" -d 'Stop a topology'
complete -c wdc -n "__fish_wdc_using_subcommand topology; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wdc -n "__fish_wdc_using_subcommand console" -s m -l management -d 'The published management port' -r
complete -c wdc -n "__fish_wdc_using_subcommand console" -s h -l help -d 'Print help'
complete -c wdc -n "__fish_wdc_using_subcommand console" -s V -l version -d 'Print version'
complete -c wdc -n "__fish_wdc_using_subcommand cli" -s m -l management -d 'The published management port' -r
complete -c wdc -n "__fish_wdc_using_subcommand cli" -s h -l help -d 'Print help'
complete -c wdc -n "__fish_wdc_using_subcommand cli" -s V -l version -d 'Print version'
complete -c wdc -n "__fish_wdc_using_subcommand help; and not __fish_seen_subcommand_from build start stop dc hc topology console cli help" -f -a "build" -d 'Build WildFly images'
complete -c wdc -n "__fish_wdc_using_subcommand help; and not __fish_seen_subcommand_from build start stop dc hc topology console cli help" -f -a "start" -d 'Start a standalone server'
complete -c wdc -n "__fish_wdc_using_subcommand help; and not __fish_seen_subcommand_from build start stop dc hc topology console cli help" -f -a "stop" -d 'Stop a standalone server'
complete -c wdc -n "__fish_wdc_using_subcommand help; and not __fish_seen_subcommand_from build start stop dc hc topology console cli help" -f -a "dc" -d 'Start and stop a domain controller'
complete -c wdc -n "__fish_wdc_using_subcommand help; and not __fish_seen_subcommand_from build start stop dc hc topology console cli help" -f -a "hc" -d 'Start and stop a host controller'
complete -c wdc -n "__fish_wdc_using_subcommand help; and not __fish_seen_subcommand_from build start stop dc hc topology console cli help" -f -a "topology" -d 'Start and stop a topology defined as YAML'
complete -c wdc -n "__fish_wdc_using_subcommand help; and not __fish_seen_subcommand_from build start stop dc hc topology console cli help" -f -a "console" -d 'Open the management console'
complete -c wdc -n "__fish_wdc_using_subcommand help; and not __fish_seen_subcommand_from build start stop dc hc topology console cli help" -f -a "cli" -d 'Connect to the CLI'
complete -c wdc -n "__fish_wdc_using_subcommand help; and not __fish_seen_subcommand_from build start stop dc hc topology console cli help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c wdc -n "__fish_wdc_using_subcommand help; and __fish_seen_subcommand_from dc" -f -a "start" -d 'Start a domain controller'
complete -c wdc -n "__fish_wdc_using_subcommand help; and __fish_seen_subcommand_from dc" -f -a "stop" -d 'Stop a domain controller'
complete -c wdc -n "__fish_wdc_using_subcommand help; and __fish_seen_subcommand_from hc" -f -a "start" -d 'Start a host controller'
complete -c wdc -n "__fish_wdc_using_subcommand help; and __fish_seen_subcommand_from hc" -f -a "stop" -d 'Stop a host controller'
complete -c wdc -n "__fish_wdc_using_subcommand help; and __fish_seen_subcommand_from topology" -f -a "start" -d 'Start a topology'
complete -c wdc -n "__fish_wdc_using_subcommand help; and __fish_seen_subcommand_from topology" -f -a "stop" -d 'Stop a topology'
