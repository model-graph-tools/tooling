
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'mgt' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'mgt'
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
        'mgt' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('analyze', 'analyze', [CompletionResultType]::ParameterValue, 'Analyze the management model of a WildFly instance and build an image with a Neo4J database')
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start Neo4J model DB containers')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop Neo4J model DB containers')
            [CompletionResult]::new('browse', 'browse', [CompletionResultType]::ParameterValue, 'Open the Neo4J browser for a running Neo4J model DB')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'mgt;analyze' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'mgt;start' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'mgt;stop' {
            [CompletionResult]::new('-a', '-a', [CompletionResultType]::ParameterName, 'Stop all running Neo4J model DB containers')
            [CompletionResult]::new('--all', '--all', [CompletionResultType]::ParameterName, 'Stop all running Neo4J model DB containers')
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'mgt;browse' {
            [CompletionResult]::new('-h', '-h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', '--help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', '-V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', '--version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
        'mgt;help' {
            [CompletionResult]::new('analyze', 'analyze', [CompletionResultType]::ParameterValue, 'Analyze the management model of a WildFly instance and build an image with a Neo4J database')
            [CompletionResult]::new('start', 'start', [CompletionResultType]::ParameterValue, 'Start Neo4J model DB containers')
            [CompletionResult]::new('stop', 'stop', [CompletionResultType]::ParameterValue, 'Stop Neo4J model DB containers')
            [CompletionResult]::new('browse', 'browse', [CompletionResultType]::ParameterValue, 'Open the Neo4J browser for a running Neo4J model DB')
            [CompletionResult]::new('help', 'help', [CompletionResultType]::ParameterValue, 'Print this message or the help of the given subcommand(s)')
            break
        }
        'mgt;help;analyze' {
            break
        }
        'mgt;help;start' {
            break
        }
        'mgt;help;stop' {
            break
        }
        'mgt;help;browse' {
            break
        }
        'mgt;help;help' {
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
