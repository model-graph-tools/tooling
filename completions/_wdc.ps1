
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'wdc' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'wdc'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'wdc' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('build', 'build', [CompletionResultType]::ParameterValue, 'Build WildFly images')
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start a standalone server')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop a standalone server')
            [CompletionResult]::new('dc', 'dc', [CompletionResultType]::ParameterValue, 'Start and stop a domain controller')
            [CompletionResult]::new('hc', 'hc', [CompletionResultType]::ParameterValue, 'Start and stop a host controller')
            [CompletionResult]::new('topology', 'topology', [CompletionResultType]::ParameterValue, 'Start and stop a topology defined as YAML')
            [CompletionResult]::new('console', 'console', [CompletionResultType]::ParameterValue, 'Open the management console')
            [CompletionResult]::new('cli', 'cli', [CompletionResultType]::ParameterValue, 'Connect to the CLI')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'wdc;build' {
            [CompletionResult]::new('-u', '-u', [CompletionResultType]::ParameterName, 'The username of the management user')
            [CompletionResult]::new('--username', '--username', [CompletionResultType]::ParameterName, 'The username of the management user')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'The password of the management user')
            [CompletionResult]::new('--password', '--password', [CompletionResultType]::ParameterName, 'The password of the management user')
            [CompletionResult]::new('--chunks', '--chunks', [CompletionResultType]::ParameterName, 'Build the images in chunks of this size. If not specified, the images are built in one go.')
            [CompletionResult]::new('-l', '-l', [CompletionResultType]::ParameterName, 'Mark the version as the latest version. If used with a version range, the latest tag is applied to the largest version in the range.')
            [CompletionResult]::new('--latest', '--latest', [CompletionResultType]::ParameterName, 'Mark the version as the latest version. If used with a version range, the latest tag is applied to the largest version in the range.')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'wdc;start' {
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'The name of the standalone server')
            [CompletionResult]::new('--name', '--name', [CompletionResultType]::ParameterName, 'The name of the standalone server')
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'The published HTTP port')
            [CompletionResult]::new('--http', '--http', [CompletionResultType]::ParameterName, 'The published HTTP port')
            [CompletionResult]::new('-m', '-m', [CompletionResultType]::ParameterName, 'The published management port')
            [CompletionResult]::new('--management', '--management', [CompletionResultType]::ParameterName, 'The published management port')
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'The offset added to the published HTTP and management ports')
            [CompletionResult]::new('--offset', '--offset', [CompletionResultType]::ParameterName, 'The offset added to the published HTTP and management ports')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'wdc;stop' {
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'The name of the standalone server')
            [CompletionResult]::new('--name', '--name', [CompletionResultType]::ParameterName, 'The name of the standalone server')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'wdc;dc' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start a domain controller')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop a domain controller')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'wdc;dc;start' {
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'The name of the domain controller')
            [CompletionResult]::new('--name', '--name', [CompletionResultType]::ParameterName, 'The name of the domain controller')
            [CompletionResult]::new('-p', '-p', [CompletionResultType]::ParameterName, 'The published HTTP port')
            [CompletionResult]::new('--http', '--http', [CompletionResultType]::ParameterName, 'The published HTTP port')
            [CompletionResult]::new('-m', '-m', [CompletionResultType]::ParameterName, 'The published management port')
            [CompletionResult]::new('--management', '--management', [CompletionResultType]::ParameterName, 'The published management port')
            [CompletionResult]::new('-o', '-o', [CompletionResultType]::ParameterName, 'The offset added to the published HTTP and management ports')
            [CompletionResult]::new('--offset', '--offset', [CompletionResultType]::ParameterName, 'The offset added to the published HTTP and management ports')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Manage servers of the domain controller. Servers are specified as <name>[:<server-group>][:<offset>][:start][:publish] <name>          The name of the server. This part is mandatory and must be first.                 All other parts are optional and can follow in any order. <server-group>  The name of the server group. Allowed values are ''main-server-group'' or ''msg'',                 and ''other-server-group'' or ''osg''. If not specified, ''main-server-group'' is used. <offset>        The port offset. If not specified, 100 is used from the second server onwards. start           Whether to start the server. publish         Whether to publish the HTTP port.')
            [CompletionResult]::new('--server', '--server', [CompletionResultType]::ParameterName, 'Manage servers of the domain controller. Servers are specified as <name>[:<server-group>][:<offset>][:start][:publish] <name>          The name of the server. This part is mandatory and must be first.                 All other parts are optional and can follow in any order. <server-group>  The name of the server group. Allowed values are ''main-server-group'' or ''msg'',                 and ''other-server-group'' or ''osg''. If not specified, ''main-server-group'' is used. <offset>        The port offset. If not specified, 100 is used from the second server onwards. start           Whether to start the server. publish         Whether to publish the HTTP port.')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'wdc;dc;stop' {
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'The name of the domain controller')
            [CompletionResult]::new('--name', '--name', [CompletionResultType]::ParameterName, 'The name of the domain controller')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'wdc;dc;help' {
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start a domain controller')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop a domain controller')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'wdc;dc;help;start' {
            break
        }
        'wdc;dc;help;stop' {
            break
        }
        'wdc;dc;help;help' {
            break
        }
        'wdc;hc' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start a host controller')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop a host controller')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'wdc;hc;start' {
            [CompletionResult]::new('-d', '-d', [CompletionResultType]::ParameterName, 'The name of the domain controller')
            [CompletionResult]::new('--domain-controller', '--domain-controller', [CompletionResultType]::ParameterName, 'The name of the domain controller')
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'The name of the host controller')
            [CompletionResult]::new('--name', '--name', [CompletionResultType]::ParameterName, 'The name of the host controller')
            [CompletionResult]::new('-s', '-s', [CompletionResultType]::ParameterName, 'Manage servers of the host controller. Servers are specified as <name>[:<server-group>][:<offset>][:start][:publish] <name>          The name of the server. This part is mandatory and must be first.                 All other parts are optional and can follow in any order. <server-group>  The name of the server group. Allowed values are ''main-server-group'' or ''msg'',                 and ''other-server-group'' or ''osg''. If not specified, ''main-server-group'' is used. <offset>        The port offset. If not specified, 100 is used from the second server onwards. start           Whether to start the server. publish         Whether to publish the HTTP port.')
            [CompletionResult]::new('--server', '--server', [CompletionResultType]::ParameterName, 'Manage servers of the host controller. Servers are specified as <name>[:<server-group>][:<offset>][:start][:publish] <name>          The name of the server. This part is mandatory and must be first.                 All other parts are optional and can follow in any order. <server-group>  The name of the server group. Allowed values are ''main-server-group'' or ''msg'',                 and ''other-server-group'' or ''osg''. If not specified, ''main-server-group'' is used. <offset>        The port offset. If not specified, 100 is used from the second server onwards. start           Whether to start the server. publish         Whether to publish the HTTP port.')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'wdc;hc;stop' {
            [CompletionResult]::new('-n', '-n', [CompletionResultType]::ParameterName, 'The name of the host controller')
            [CompletionResult]::new('--name', '--name', [CompletionResultType]::ParameterName, 'The name of the host controller')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'wdc;hc;help' {
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start a host controller')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop a host controller')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'wdc;hc;help;start' {
            break
        }
        'wdc;hc;help;stop' {
            break
        }
        'wdc;hc;help;help' {
            break
        }
        'wdc;topology' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start a topology')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop a topology')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'wdc;topology;start' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'wdc;topology;stop' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'wdc;topology;help' {
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start a topology')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop a topology')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'wdc;topology;help;start' {
            break
        }
        'wdc;topology;help;stop' {
            break
        }
        'wdc;topology;help;help' {
            break
        }
        'wdc;console' {
            [CompletionResult]::new('-m', '-m', [CompletionResultType]::ParameterName, 'The published management port')
            [CompletionResult]::new('--management', '--management', [CompletionResultType]::ParameterName, 'The published management port')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'wdc;cli' {
            [CompletionResult]::new('-m', '-m', [CompletionResultType]::ParameterName, 'The published management port')
            [CompletionResult]::new('--management', '--management', [CompletionResultType]::ParameterName, 'The published management port')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'wdc;help' {
            [CompletionResult]::new('build', 'build', [CompletionResultType]::ParameterValue, 'Build WildFly images')
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start a standalone server')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop a standalone server')
            [CompletionResult]::new('dc', 'dc', [CompletionResultType]::ParameterValue, 'Start and stop a domain controller')
            [CompletionResult]::new('hc', 'hc', [CompletionResultType]::ParameterValue, 'Start and stop a host controller')
            [CompletionResult]::new('topology', 'topology', [CompletionResultType]::ParameterValue, 'Start and stop a topology defined as YAML')
            [CompletionResult]::new('console', 'console', [CompletionResultType]::ParameterValue, 'Open the management console')
            [CompletionResult]::new('cli', 'cli', [CompletionResultType]::ParameterValue, 'Connect to the CLI')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'wdc;help;build' {
            break
        }
        'wdc;help;start' {
            break
        }
        'wdc;help;stop' {
            break
        }
        'wdc;help;dc' {
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start a domain controller')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop a domain controller')
            break
        }
        'wdc;help;dc;start' {
            break
        }
        'wdc;help;dc;stop' {
            break
        }
        'wdc;help;hc' {
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start a host controller')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop a host controller')
            break
        }
        'wdc;help;hc;start' {
            break
        }
        'wdc;help;hc;stop' {
            break
        }
        'wdc;help;topology' {
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start a topology')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop a topology')
            break
        }
        'wdc;help;topology;start' {
            break
        }
        'wdc;help;topology;stop' {
            break
        }
        'wdc;help;console' {
            break
        }
        'wdc;help;cli' {
            break
        }
        'wdc;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
