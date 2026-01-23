@{
    # Module manifest for NudgePredictor
    # LLM-powered command prediction for PowerShell 7.2+

    # Script module file
    RootModule = 'NudgePredictor.psm1'

    # Version number
    ModuleVersion = '0.3.0'

    # Unique identifier
    GUID = 'a1b2c3d4-e5f6-7890-abcd-ef1234567890'

    # Author
    Author = 'Nudge Team'

    # Company
    CompanyName = 'Nudge'

    # Copyright
    Copyright = '(c) 2024 Nudge Team. All rights reserved.'

    # Description
    Description = 'LLM-powered command prediction for PowerShell using PSReadLine predictor API'

    # Minimum PowerShell version
    PowerShellVersion = '7.2'

    # Required modules
    RequiredModules = @(
        @{ ModuleName = 'PSReadLine'; ModuleVersion = '2.2.0' }
    )

    # Functions to export
    FunctionsToExport = @(
        'Register-NudgePredictor',
        'Unregister-NudgePredictor',
        'Set-NudgePredictionOptions'
    )

    # Cmdlets to export
    CmdletsToExport = @()

    # Variables to export
    VariablesToExport = @()

    # Aliases to export
    AliasesToExport = @()

    # Private data
    PrivateData = @{
        PSData = @{
            # Tags for PowerShell Gallery
            Tags = @('Prediction', 'Completion', 'LLM', 'AI', 'PSReadLine', 'CommandPredictor')

            # License URI
            LicenseUri = 'https://github.com/Zhangtiande/nudge/blob/main/LICENSE'

            # Project URI
            ProjectUri = 'https://github.com/Zhangtiande/nudge'

            # Icon URI
            # IconUri = ''

            # Release notes
            ReleaseNotes = @'
## Version 0.2.1
- Initial release of NudgePredictor module
- Implements ICommandPredictor for PSReadLine integration
- Provides LLM-powered command suggestions
- Includes caching and throttling for performance
- Requires PowerShell 7.2+ and PSReadLine 2.2.0+
'@

            # Prerelease string
            # Prerelease = ''

            # External module dependencies
            ExternalModuleDependencies = @()
        }
    }

    # HelpInfo URI
    # HelpInfoURI = ''
}
