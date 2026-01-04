using StackExchange.Redis;
using System;

class Program
{
    static void Main(string[] args)
    {
        var redis = ConnectionMultiplexer.Connect("localhost");
        var db = redis.GetDatabase();

        Console.WriteLine("--- 03 JSON Document Store ---");

        string json = "{ \"id\": 101, \"title\": \"Zedis Guide\", \"tags\": [\"db\", \"rust\"] }";

        // JSON.SET
        Console.WriteLine("Storing JSON...");
        db.Execute("JSON.SET", "post:101", json);

        // JSON.GET
        Console.WriteLine("Retrieving JSON...");
        var data = db.Execute("JSON.GET", "post:101", ".");
        Console.WriteLine($"Result: {data}");
    }
}
