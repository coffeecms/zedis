using StackExchange.Redis;
using System;

class Program
{
    static void Main(string[] args)
    {
        var redis = ConnectionMultiplexer.Connect("localhost");
        var db = redis.GetDatabase();

        Console.WriteLine("--- 07 Complex Structures ---");

        // List
        db.ListRightPush("logs", "Error: File not found");
        db.ListRightPush("logs", "Info: User logged in");
        Console.WriteLine($"Logs count: {db.ListLength("logs")}");

        // Set
        db.SetAdd("admins", "user:1");
        db.SetAdd("admins", "user:2");
        bool isAdmin = db.SetContains("admins", "user:1");
        Console.WriteLine($"Is user:1 admin? {isAdmin}");

        // Hash
        db.HashSet("user:1", new HashEntry[] { 
            new HashEntry("name", "Alice"), 
            new HashEntry("age", 30) 
        });
        string name = db.HashGet("user:1", "name");
        Console.WriteLine($"User Name: {name}");
    }
}
