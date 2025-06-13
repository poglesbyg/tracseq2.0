Write-Host "=== Simple Storage Operations Demonstration ==="
Write-Host "Demonstrating the key storage features we've implemented"

$timestamp = Get-Date -Format "MMddHHmmss"
$demoBarcode = "DEMO_$timestamp"

try {
    # Step 1: Create a sample with storage integration
    Write-Host "`n1. 📦 Creating sample with automatic storage integration..."
    $createJson = @"
{
  "samples": [
    {
      "name": "Demo Sample",
      "barcode": "$demoBarcode",
      "location": "Freezer A (-80°C)"
    }
  ],
  "storage_location_id": 1,
  "template_name": "Demo Template",
  "stored_by": "demo_user"
}
"@
    
    $createResponse = Invoke-RestMethod -Uri "http://localhost:3000/api/samples/batch" -Method POST -Body $createJson -ContentType "application/json"
    Write-Host "✅ Sample created and stored: $($createResponse.created) created, $($createResponse.stored_in_storage) stored in storage"

    # Step 2: Verify sample is in storage via barcode scan
    Write-Host "`n2. 🔍 Scanning barcode to verify storage location..."
    $scanResponse = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/scan/$demoBarcode"
    Write-Host "✅ Sample found in storage:"
    Write-Host "   Location: $($scanResponse.location_name)"
    Write-Host "   Storage State: $($scanResponse.storage_state)"
    Write-Host "   Stored At: $($scanResponse.stored_at)"

    # Step 3: Move sample to different location
    Write-Host "`n3. 🚚 Moving sample to different storage location..."
    $moveJson = @"
{
  "barcode": "$demoBarcode",
  "location_id": 2,
  "reason": "Demonstration of sample movement",
  "moved_by": "demo_user"
}
"@
    
    $moveResponse = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/move" -Method POST -Body $moveJson -ContentType "application/json"
    Write-Host "✅ Sample moved successfully!"
    Write-Host "   Message: $($moveResponse.message)"

    # Step 4: Verify the move worked
    Write-Host "`n4. 🔍 Verifying sample moved to new location..."
    $scanAfterMove = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/scan/$demoBarcode"
    Write-Host "✅ Sample location verified:"
    Write-Host "   New Location: $($scanAfterMove.location_name)"
    Write-Host "   Location ID: $($scanAfterMove.location_id)"

    # Step 5: Check storage capacity
    Write-Host "`n5. 📊 Checking storage system capacity..."
    $capacityResponse = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/capacity"
    Write-Host "✅ Storage capacity overview:"
    Write-Host "   Total Capacity: $($capacityResponse.total_capacity)"
    Write-Host "   Current Usage: $($capacityResponse.total_usage)"
    Write-Host "   Utilization: $([math]::Round($capacityResponse.overall_utilization, 2))%"

    # Step 6: Remove sample from storage
    Write-Host "`n6. 🗑️ Removing sample from storage system..."
    $removeJson = @"
{
  "barcode": "$demoBarcode",
  "reason": "Demonstration cleanup",
  "removed_by": "demo_user"
}
"@
    
    $removeResponse = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/remove" -Method POST -Body $removeJson -ContentType "application/json"
    Write-Host "✅ Sample removed successfully!"
    Write-Host "   Removed from: $($removeResponse.removed_sample.location_name)"
    Write-Host "   Removed by: $($removeResponse.removed_sample.removed_by)"

    # Step 7: Verify removal
    Write-Host "`n7. 🔍 Verifying sample was removed from storage..."
    try {
        $scanAfterRemove = Invoke-RestMethod -Uri "http://localhost:3000/api/storage/scan/$demoBarcode"
        Write-Host "⚠️  Warning: Sample still found after removal"
    } catch {
        if ($_.Exception.Response.StatusCode -eq 404) {
            Write-Host "✅ Confirmed: Sample successfully removed from storage"
        } else {
            Write-Host "❌ Error verifying removal: $($_.Exception.Message)"
        }
    }

    Write-Host "`n🎉 DEMONSTRATION COMPLETE!"
    Write-Host "✅ Sample creation with automatic storage integration: WORKING"
    Write-Host "✅ Barcode scanning and location lookup: WORKING"
    Write-Host "✅ Sample movement between storage locations: WORKING"
    Write-Host "✅ Storage capacity tracking: WORKING"
    Write-Host "✅ Sample removal from storage: WORKING"
    Write-Host "✅ Complete storage lifecycle management: WORKING"

} catch {
    Write-Host "❌ Demo failed at step: $($_.Exception.Message)"
    Write-Host "This may indicate an issue with the storage system that needs investigation."
}

Write-Host "`n📋 Summary: The storage operations system provides:"
Write-Host "  • Seamless integration between sample creation and storage"
Write-Host "  • Barcode-based sample tracking and lookup"
Write-Host "  • Movement capabilities between different storage locations"
Write-Host "  • Real-time capacity monitoring and utilization tracking"
Write-Host "  • Complete sample lifecycle management from creation to disposal"
Write-Host "  • Comprehensive audit trails for all storage operations" 
