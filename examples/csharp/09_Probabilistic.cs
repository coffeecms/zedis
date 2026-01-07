using StackExchange.Redis;
using System;

class Program
{
    static void Main()
    {
        var redis = ConnectionMultiplexer.Connect("localhost");
        var db = redis.GetDatabase();

        Console.WriteLine("--- 09 God Tier Probabilistic Data Structures ---");

        // 1. HyperLogLog
        db.Execute("PFADD", "hll_cs", "a", "b", "c");
        var count = db.Execute("PFCOUNT", "hll_cs");
        Console.WriteLine($"[HLL] Count: {count}");

        // 2. Cuckoo Filter
        db.Execute("CF.ADD", "cf_cs", "test");
        var exists = db.Execute("CF.EXISTS", "cf_cs", "test");
        Console.WriteLine($"[Cuckoo] Exists: {exists}");

        // 3. Count-Min Sketch
        db.Execute("CMS.INCRBY", "cms_cs", "eventA", 100);
        var q = db.Execute("CMS.QUERY", "cms_cs", "eventA");
        Console.WriteLine($"[CMS] Query: {q}");

        // 4. Top-K
        db.Execute("TOPK.ADD", "topk_cs", "item1", "item2", "item1");
        var list = db.Execute("TOPK.LIST", "topk_cs");
        Console.WriteLine($"[TopK] items: {list}");

        // 5. t-digest
        db.Execute("TDIGEST.ADD", "td_cs", 99.9);
        var p50 = db.Execute("TDIGEST.QUANTILE", "td_cs", 0.5);
        Console.WriteLine($"[t-digest] Quantile: {p50}");
    }
}
