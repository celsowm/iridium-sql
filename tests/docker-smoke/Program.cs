using Microsoft.Data.SqlClient;

var port = args.Length > 0 ? args[0] : "14330";
var connectionString = $"Server=127.0.0.1,{port};Database=master;User Id=sa;Password=test-only;Encrypt=True;TrustServerCertificate=True;Connection Timeout=10;";

await using var connection = new SqlConnection(connectionString);
await connection.OpenAsync();
await using var command = new SqlCommand("SELECT 1", connection);
var actual = await command.ExecuteScalarAsync();
if (Convert.ToInt32(actual) != 1)
{
    throw new Exception($"Expected 1 from Iridium SQL, got {actual}");
}

Console.WriteLine($"Iridium SQL TDS smoke test passed on port {port}.");
