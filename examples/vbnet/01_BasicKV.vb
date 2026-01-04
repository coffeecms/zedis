Imports StackExchange.Redis
Imports System

Module Program
    Sub Main(args As String())
        Dim redis = ConnectionMultiplexer.Connect("localhost")
        Dim db = redis.GetDatabase()

        Console.WriteLine("--- 01 Basic Key-Value Operations ---")

        ' SET
        Console.WriteLine("Setting key 'status' to 'active'...")
        db.StringSet("status", "active")

        ' GET
        Dim value = db.StringGet("status")
        Console.WriteLine($"Got value: {value}")

        ' EXPIRY
        Console.WriteLine("Setting 'temp_key' with 10s expiry...")
        db.StringSet("temp_key", "123", TimeSpan.FromSeconds(10))

        ' PING
        Dim latency = db.Ping()
        Console.WriteLine($"Ping: {latency.TotalMilliseconds} ms")
    End Sub
End Module
