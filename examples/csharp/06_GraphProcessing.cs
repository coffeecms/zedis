using StackExchange.Redis;
using System;

class Program
{
    static void Main(string[] args)
    {
        var redis = ConnectionMultiplexer.Connect("localhost");
        var db = redis.GetDatabase();

        Console.WriteLine("--- 06 Graph Processing ---");
        string key = "cities";

        // GRAPH.ADD
        db.Execute("GRAPH.ADD", key, "Berlin", "Munich");
        db.Execute("GRAPH.ADD", key, "Munich", "Vienna");
        db.Execute("GRAPH.ADD", key, "Berlin", "Hamburg");

        // GRAPH.BFS
        Console.WriteLine("Cities reachable from Berlin (Depth 2):");
        var result = db.Execute("GRAPH.BFS", key, "Berlin", 2);
        Console.WriteLine(result.ToString());
    }
}
