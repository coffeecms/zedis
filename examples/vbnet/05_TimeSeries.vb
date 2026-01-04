Imports StackExchange.Redis
Imports System

Module Program
    Sub Main(args As String())
        Dim redis = ConnectionMultiplexer.Connect("localhost")
        Dim db = redis.GetDatabase()

        Console.WriteLine("--- 05 Time Series ---")
        Dim key As String = "metrics:load"
        Dim now As Long = DateTimeOffset.UtcNow.ToUnixTimeMilliseconds()

        ' TS.ADD
        db.Execute("TS.ADD", key, now, 55)
        db.Execute("TS.ADD", key, now + 1000, 60)

        ' TS.RANGE
        Console.WriteLine("Querying metrics...")
        Dim data = db.Execute("TS.RANGE", key, now, now + 5000)
        Console.WriteLine(data.ToString())
    End Sub
End Module
