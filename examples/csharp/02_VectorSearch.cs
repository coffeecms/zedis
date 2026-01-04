using StackExchange.Redis;
using System;

class Program
{
    static void Main(string[] args)
    {
        var redis = ConnectionMultiplexer.Connect("localhost");
        var db = redis.GetDatabase();

        Console.WriteLine("--- 02 AI Vector Search (Auto-Embedding) ---");

        // 1. Ingest Data
        Console.WriteLine("Ingesting support tickets...");
        db.Execute("VADD.TEXT", "ticket:1", "My screen flickers when I type");
        db.Execute("VADD.TEXT", "ticket:2", "Battery drains very fast on standby");
        db.Execute("VADD.TEXT", "ticket:3", "Cannot connect to WiFi network");

        // 2. Search
        string query = "display issues";
        Console.WriteLine($"\nSearching for: '{query}'");

        // VSEARCH.TEXT <index> <query> <limit>
        var result = db.Execute("VSEARCH.TEXT", "ticket", query, 2);
        
        // Output results (RedisResult needs parsing based on structure)
        Console.WriteLine("Results: " + result.ToString());
    }
}
