Imports StackExchange.Redis
Imports System

Module Program
    Sub Main(args As String())
        Dim redis = ConnectionMultiplexer.Connect("localhost")
        Dim db = redis.GetDatabase()

        Console.WriteLine("--- 02 AI Vector Search (Auto-Embedding) ---")

        ' 1. Ingest
        Console.WriteLine("Ingesting messages...")
        db.Execute("VADD.TEXT", "msg:1", "Hello world, this is a test")
        db.Execute("VADD.TEXT", "msg:2", "System breakdown imminent")
        db.Execute("VADD.TEXT", "msg:3", "Greetings earthlings")

        ' 2. Search
        Dim query As String = "hi planet"
        Console.WriteLine($"Searching for: '{query}'")

        ' VSEARCH.TEXT <index> <query> <limit>
        Dim result = db.Execute("VSEARCH.TEXT", "msg", query, 2)
        Console.WriteLine("Results: " & result.ToString())
    End Sub
End Module
