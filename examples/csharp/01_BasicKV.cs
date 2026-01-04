using StackExchange.Redis;
using System;

class Program
{
    static void Main(string[] args)
    {
        ConnectionMultiplexer redis = ConnectionMultiplexer.Connect("localhost");
        IDatabase db = redis.GetDatabase();

        Console.WriteLine("--- 01 Basic Key-Value Operations ---");

        // SET
        Console.WriteLine("Setting key 'username' to 'john_doe'...");
        db.StringSet("username", "john_doe");

        // GET
        string value = db.StringGet("username");
        Console.WriteLine($"Got value: {value}");

        // EXPIRY
        Console.WriteLine("Setting 'otp' with 30s expiry...");
        db.StringSet("otp", "5555", TimeSpan.FromSeconds(30));

        // PING (Latency Check)
        var latency = db.Ping();
        Console.WriteLine($"Ping: {latency.TotalMilliseconds} ms");
    }
}
