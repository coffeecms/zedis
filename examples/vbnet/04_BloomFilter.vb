Imports StackExchange.Redis
Imports System

Module Program
    Sub Main(args As String())
        Dim redis = ConnectionMultiplexer.Connect("localhost")
        Dim db = redis.GetDatabase()

        Console.WriteLine("--- 04 Bloom Filter ---")
        Dim key As String = "banned_users"

        ' BF.ADD
        db.Execute("BF.ADD", key, "user_A")
        db.Execute("BF.ADD", key, "user_B")

        ' BF.EXISTS
        Dim exists As Boolean = CInt(db.Execute("BF.EXISTS", key, "user_A")) = 1
        Console.WriteLine($"Is user_A banned? {exists}")
    End Sub
End Module
