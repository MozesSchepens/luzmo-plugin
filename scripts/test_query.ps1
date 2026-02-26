$secret = "T95bgRqI1bSHobi91hQUO8CvJ"
$url = "http://localhost:3000/query"

Write-Host "TEST 1: Minimal Query" -ForegroundColor Cyan
$body1 = '{"dataset_id":"demo"}'
try {
    $resp = Invoke-RestMethod -Method Post $url -Headers @{"X-Secret"=$secret;"Content-Type"="application/json"} -Body $body1 
    Write-Host "SUCCESS - Row count: $($resp.Count)" -ForegroundColor Green
} catch {
    Write-Host "FAILED: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "`nTEST 2: Query with raw columns" -ForegroundColor Cyan
$body2 = '{"dataset_id":"demo","columns":[{"id":"category"},{"id":"date"},{"id":"value"}],"limit":3}'
try {
    $resp = Invoke-RestMethod -Method Post $url -Headers @{"X-Secret"=$secret;"Content-Type"="application/json"} -Body $body2
    Write-Host "SUCCESS - Row count: $($resp.Count)" -ForegroundColor Green
    if ($resp.value) { Write-Host "First row: $(($resp.value[0] | ConvertTo-Json -Compress))" }
} catch {
    Write-Host "FAILED: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "`nTEST 3: Query with aggregation" -ForegroundColor Cyan
$body3 = '{"dataset_id":"demo","columns":[{"id":"category"},{"id":"value","aggregation":"sum"}]}'
try {
    $resp = Invoke-RestMethod -Method Post $url -Headers @{"X-Secret"=$secret;"Content-Type"="application/json"} -Body $body3
    Write-Host "SUCCESS - Row count: $($resp.Count)" -ForegroundColor Green
    if ($resp.value) { Write-Host "Result: $(($resp.value | ConvertTo-Json -Compress))" }
} catch {
    Write-Host "FAILED: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host "`nTEST 4: Query with date grouping by month" -ForegroundColor Cyan
$body4 = '{"dataset_id":"demo","columns":[{"id":"date","level":"month"},{"id":"value","aggregation":"sum"}],"limit":3}'
try {
    $resp = Invoke-RestMethod -Method Post $url -Headers @{"X-Secret"=$secret;"Content-Type"="application/json"} -Body $body4
    Write-Host "SUCCESS - Row count: $($resp.Count)" -ForegroundColor Green
    if ($resp.value) { Write-Host "Results: $(($resp.value | ConvertTo-Json -Compress))" }
} catch {
    Write-Host "FAILED: $($_.Exception.Message)" -ForegroundColor Red
}
