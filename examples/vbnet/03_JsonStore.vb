Imports StackExchange.Redis
Imports System

Module Program
    Sub Main(args As String())
        Dim redis = ConnectionMultiplexer.Connect("localhost")
        Dim db = redis.GetDatabase()

        Console.WriteLine("--- 03 JSON Document Store ---")

        Dim json As String = "{ ""name"": ""VB.NET"", ""type"": ""language"" }"

        ' JSON.SET
        Console.WriteLine("Storing JSON...")
        db.Execute("JSON.SET", "lang:vb", json)

        ' JSON.GET
        Console.WriteLine("Retrieving JSON...")
        Dim data = db.Execute("JSON.GET", "lang:vb", ".")
        Console.WriteLine($"Result: {data}")
    End Sub
End Module
