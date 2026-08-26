param(
    [string]$Repo = "Tha1s/desktop-widgets",
    [string]$Branch = "feat/desktop-hub-engine",
    [string]$Artifact = "workerw-proof",
    [int]$Run = -1
)
# Récupère un artefact de build GitHub Actions via gh CLI (pas de navigateur,
# donc pas de MOTW). Prérequis : gh installé (winget install --id GitHub.cli)
# et authentifié (gh auth login).
if ($Run -eq -1) {
    $Run = gh run list --branch $Branch --limit 1 --json databaseId -q '.[0].databaseId'
    if (-not $Run) { Write-Error "Aucun run trouvé sur $Branch"; exit 1 }
}
Write-Host "Téléchargement de l'artefact '$Artifact' du run $Run..."
gh run download $Run -n $Artifact -R $Repo
Write-Host "Extrait dans .\$Artifact\ — lancement : .\$Artifact\$Artifact.exe"
