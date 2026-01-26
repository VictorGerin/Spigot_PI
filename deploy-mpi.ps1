# Configurações
$ImageName = "spigot-mpi:latest"
$TarFile = "spigot-mpi.tar"
$ClusterNodes = kubectl get nodes -o jsonpath='{.items[*].metadata.name}'
$JobFile = "spigot-job.yaml"

Write-Host "🚀 Iniciando ciclo de Deploy MPI..." -ForegroundColor Cyan

# 1. Build da Imagem
Write-Host "`n📦 Construindo imagem Docker..." -ForegroundColor Yellow
docker build -t $ImageName .
if ($LASTEXITCODE -ne 0) { Write-Error "Falha no Build"; exit 1 }

# 2. Salvar Imagem em TAR
Write-Host "`n💾 Salvando imagem para arquivo temporário..." -ForegroundColor Yellow
docker save $ImageName -o $TarFile

# 3. Injetar nos Nós (Kind/Docker)
$NodesArray = $ClusterNodes -split " "
foreach ($Node in $NodesArray) {
    Write-Host "💉 Injetando imagem no nó: $Node" -ForegroundColor Magenta
    
    # Copia o arquivo .tar para dentro do container do nó
    docker cp $TarFile "${Node}:/${TarFile}"
    
    # Importa usando o containerd (ctr) de dentro do nó
    # Nota: Usamos ctr -n k8s.io images import para garantir que o K8s enxergue
    docker exec $Node ctr -n k8s.io images import "/${TarFile}" | Out-Null
    
    # Limpa o arquivo temporário dentro do nó
    docker exec $Node rm "/${TarFile}"
}

# 4. Limpeza local
Remove-Item $TarFile

# 5. Redeploy no Kubernetes
Write-Host "`n🔄 Reiniciando Job no Kubernetes..." -ForegroundColor Yellow
kubectl delete -f $JobFile --ignore-not-found=$true
kubectl apply -f $JobFile

Write-Host "`n✅ Deploy concluído! Acompanhe o launcher com:" -ForegroundColor Green
Write-Host "kubectl get pods"
Write-Host "kubectl logs -f spigot-pi-job-launcher-xxxxx"
