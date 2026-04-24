
use builtin;
use str;

set edit:completion:arg-completer[wdc] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'wdc'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'wdc'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand build 'Build WildFly images'
            cand start 'Start a standalone server'
            cand stop 'Stop a standalone server'
            cand dc 'Start and stop a domain controller'
            cand hc 'Start and stop a host controller'
            cand topology 'Start and stop a topology defined as YAML'
            cand console 'Open the management console'
            cand cli 'Connect to the CLI'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'wdc;build'= {
            cand -u 'The username of the management user'
            cand --username 'The username of the management user'
            cand -s 'The password of the management user'
            cand --password 'The password of the management user'
            cand --chunks 'Build the images in chunks of this size. If not specified, the images are built in one go.'
            cand -l 'Mark the version as the latest version. If used with a version range, the latest tag is applied to the largest version in the range.'
            cand --latest 'Mark the version as the latest version. If used with a version range, the latest tag is applied to the largest version in the range.'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'wdc;start'= {
            cand -n 'The name of the standalone server'
            cand --name 'The name of the standalone server'
            cand -p 'The published HTTP port'
            cand --http 'The published HTTP port'
            cand -m 'The published management port'
            cand --management 'The published management port'
            cand -o 'The offset added to the published HTTP and management ports'
            cand --offset 'The offset added to the published HTTP and management ports'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'wdc;stop'= {
            cand -n 'The name of the standalone server'
            cand --name 'The name of the standalone server'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'wdc;dc'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand start 'Start a domain controller'
            cand stop 'Stop a domain controller'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'wdc;dc;start'= {
            cand -n 'The name of the domain controller'
            cand --name 'The name of the domain controller'
            cand -p 'The published HTTP port'
            cand --http 'The published HTTP port'
            cand -m 'The published management port'
            cand --management 'The published management port'
            cand -o 'The offset added to the published HTTP and management ports'
            cand --offset 'The offset added to the published HTTP and management ports'
            cand -s 'Manage servers of the domain controller. Servers are specified as <name>[:<server-group>][:<offset>][:start][:publish] <name>          The name of the server. This part is mandatory and must be first.                 All other parts are optional and can follow in any order. <server-group>  The name of the server group. Allowed values are ''main-server-group'' or ''msg'',                 and ''other-server-group'' or ''osg''. If not specified, ''main-server-group'' is used. <offset>        The port offset. If not specified, 100 is used from the second server onwards. start           Whether to start the server. publish         Whether to publish the HTTP port.'
            cand --server 'Manage servers of the domain controller. Servers are specified as <name>[:<server-group>][:<offset>][:start][:publish] <name>          The name of the server. This part is mandatory and must be first.                 All other parts are optional and can follow in any order. <server-group>  The name of the server group. Allowed values are ''main-server-group'' or ''msg'',                 and ''other-server-group'' or ''osg''. If not specified, ''main-server-group'' is used. <offset>        The port offset. If not specified, 100 is used from the second server onwards. start           Whether to start the server. publish         Whether to publish the HTTP port.'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'wdc;dc;stop'= {
            cand -n 'The name of the domain controller'
            cand --name 'The name of the domain controller'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'wdc;dc;help'= {
            cand start 'Start a domain controller'
            cand stop 'Stop a domain controller'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'wdc;dc;help;start'= {
        }
        &'wdc;dc;help;stop'= {
        }
        &'wdc;dc;help;help'= {
        }
        &'wdc;hc'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand start 'Start a host controller'
            cand stop 'Stop a host controller'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'wdc;hc;start'= {
            cand -d 'The name of the domain controller'
            cand --domain-controller 'The name of the domain controller'
            cand -n 'The name of the host controller'
            cand --name 'The name of the host controller'
            cand -s 'Manage servers of the host controller. Servers are specified as <name>[:<server-group>][:<offset>][:start][:publish] <name>          The name of the server. This part is mandatory and must be first.                 All other parts are optional and can follow in any order. <server-group>  The name of the server group. Allowed values are ''main-server-group'' or ''msg'',                 and ''other-server-group'' or ''osg''. If not specified, ''main-server-group'' is used. <offset>        The port offset. If not specified, 100 is used from the second server onwards. start           Whether to start the server. publish         Whether to publish the HTTP port.'
            cand --server 'Manage servers of the host controller. Servers are specified as <name>[:<server-group>][:<offset>][:start][:publish] <name>          The name of the server. This part is mandatory and must be first.                 All other parts are optional and can follow in any order. <server-group>  The name of the server group. Allowed values are ''main-server-group'' or ''msg'',                 and ''other-server-group'' or ''osg''. If not specified, ''main-server-group'' is used. <offset>        The port offset. If not specified, 100 is used from the second server onwards. start           Whether to start the server. publish         Whether to publish the HTTP port.'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'wdc;hc;stop'= {
            cand -n 'The name of the host controller'
            cand --name 'The name of the host controller'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'wdc;hc;help'= {
            cand start 'Start a host controller'
            cand stop 'Stop a host controller'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'wdc;hc;help;start'= {
        }
        &'wdc;hc;help;stop'= {
        }
        &'wdc;hc;help;help'= {
        }
        &'wdc;topology'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand start 'Start a topology'
            cand stop 'Stop a topology'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'wdc;topology;start'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'wdc;topology;stop'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'wdc;topology;help'= {
            cand start 'Start a topology'
            cand stop 'Stop a topology'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'wdc;topology;help;start'= {
        }
        &'wdc;topology;help;stop'= {
        }
        &'wdc;topology;help;help'= {
        }
        &'wdc;console'= {
            cand -m 'The published management port'
            cand --management 'The published management port'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'wdc;cli'= {
            cand -m 'The published management port'
            cand --management 'The published management port'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
        &'wdc;help'= {
            cand build 'Build WildFly images'
            cand start 'Start a standalone server'
            cand stop 'Stop a standalone server'
            cand dc 'Start and stop a domain controller'
            cand hc 'Start and stop a host controller'
            cand topology 'Start and stop a topology defined as YAML'
            cand console 'Open the management console'
            cand cli 'Connect to the CLI'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'wdc;help;build'= {
        }
        &'wdc;help;start'= {
        }
        &'wdc;help;stop'= {
        }
        &'wdc;help;dc'= {
            cand start 'Start a domain controller'
            cand stop 'Stop a domain controller'
        }
        &'wdc;help;dc;start'= {
        }
        &'wdc;help;dc;stop'= {
        }
        &'wdc;help;hc'= {
            cand start 'Start a host controller'
            cand stop 'Stop a host controller'
        }
        &'wdc;help;hc;start'= {
        }
        &'wdc;help;hc;stop'= {
        }
        &'wdc;help;topology'= {
            cand start 'Start a topology'
            cand stop 'Stop a topology'
        }
        &'wdc;help;topology;start'= {
        }
        &'wdc;help;topology;stop'= {
        }
        &'wdc;help;console'= {
        }
        &'wdc;help;cli'= {
        }
        &'wdc;help;help'= {
        }
    ]
    $completions[$command]
}
