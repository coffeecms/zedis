Imports StackExchange.Redis
Imports System

Module Program
    Sub Main()
        Dim redis = ConnectionMultiplexer.Connect("localhost")
        Dim db = redis.GetDatabase()

        Console.WriteLine("--- 11 Transactions ---")

        Dim key As String = "vb_txn_key"
        db.StringSet(key, 0)

        Dim tran = db.CreateTransaction()
        tran.StringIncrementAsync(key)
        tran.StringIncrementAsync(key, 5)

        Console.WriteLine("Executing...")
        Dim committed = tran.Execute()

        Console.WriteLine($"Committed: {committed}")
        Console.WriteLine($"New Value: {db.StringGet(key)}") ' Should be 6
    End Sub
End Module
