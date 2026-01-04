using StackExchange.Redis;
using System;

class Program
{
    static void Main(string[] args)
    {
        var redis = ConnectionMultiplexer.Connect("localhost");
        var db = redis.GetDatabase();

        Console.WriteLine("--- 04 Bloom Filter ---");
        string key = "processed_transactions";

        // BF.ADD
        db.Execute("BF.ADD", key, "tx_1001");
        db.Execute("BF.ADD", key, "tx_1002");

        // BF.EXISTS
        bool exists = (int)db.Execute("BF.EXISTS", key, "tx_1001") == 1;
        Console.WriteLine($"Transaction 1001 processed? {exists}");

        bool existsNew = (int)db.Execute("BF.EXISTS", key, "tx_9999") == 1;
        Console.WriteLine($"Transaction 9999 processed? {existsNew}");
    }
}
