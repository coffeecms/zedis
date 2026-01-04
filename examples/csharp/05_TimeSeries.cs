using StackExchange.Redis;
using System;

class Program
{
    static void Main(string[] args)
    {
        var redis = ConnectionMultiplexer.Connect("localhost");
        var db = redis.GetDatabase();

        Console.WriteLine("--- 05 Time Series ---");
        string key = "sensor:voltage";

        long now = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds();

        // TS.ADD
        db.Execute("TS.ADD", key, now, 220);
        db.Execute("TS.ADD", key, now + 1000, 221);
        db.Execute("TS.ADD", key, now + 2000, 219);

        // TS.RANGE
        Console.WriteLine("Querying Time Series...");
        var data = db.Execute("TS.RANGE", key, now, now + 5000);
        Console.WriteLine(data.ToString());
    }
}
