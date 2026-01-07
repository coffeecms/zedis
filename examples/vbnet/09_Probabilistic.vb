Imports StackExchange.Redis
Imports System

Module Program
    Sub Main()
        Dim redis = ConnectionMultiplexer.Connect("localhost")
        Dim db = redis.GetDatabase()

        Console.WriteLine("--- 09 God Tier Probabilistic Data Structures ---")

        ' 1. HyperLogLog
        db.Execute("PFADD", "hll_vb", "v1", "v2")
        Dim count = db.Execute("PFCOUNT", "hll_vb")
        Console.WriteLine($"[HLL] Count: {count}")

        ' 2. Cuckoo Filter
        db.Execute("CF.ADD", "cf_vb", "itemA")
        Dim exists = db.Execute("CF.EXISTS", "cf_vb", "itemA")
        Console.WriteLine($"[Cuckoo] Exists: {exists}")

        ' 3. Count-Min Sketch
        db.Execute("CMS.INCRBY", "cms_vb", "tag1", 10)
        Dim val = db.Execute("CMS.QUERY", "cms_vb", "tag1")
        Console.WriteLine($"[CMS] Value: {val}")

        ' 4. Top-K
        db.Execute("TOPK.ADD", "topk_vb", "apple", "banana", "apple")
        Dim list = db.Execute("TOPK.LIST", "topk_vb")
        Console.WriteLine($"[TopK] List: {list}")

        ' 5. t-digest
        db.Execute("TDIGEST.ADD", "td_vb", 42.5)
        Dim res = db.Execute("TDIGEST.QUANTILE", "td_vb", 0.5)
        Console.WriteLine($"[t-digest] Median: {res}")
    End Sub
End Module
