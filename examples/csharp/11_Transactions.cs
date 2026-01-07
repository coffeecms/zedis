using StackExchange.Redis;
using System;

class Program
{
    static void Main()
    {
        var redis = ConnectionMultiplexer.Connect("localhost");
        var db = redis.GetDatabase();

        Console.WriteLine("--- 11 Transactions ---");

        var key = "cs_txn_key";
        db.StringSet(key, 0);

        var tran = db.CreateTransaction();
        
        // Enqueue commands
        var t1 = tran.StringIncrementAsync(key);
        var t2 = tran.StringIncrementAsync(key);

        Console.WriteLine("Executing Transaction...");
        bool committed = tran.Execute();
        
        Console.WriteLine($"Committed: {committed}");
        Console.WriteLine($"New Value: {db.StringGet(key)}"); // Should be 2
    }
}
