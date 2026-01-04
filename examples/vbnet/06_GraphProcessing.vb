Imports StackExchange.Redis
Imports System

Module Program
    Sub Main(args As String())
        Dim redis = ConnectionMultiplexer.Connect("localhost")
        Dim db = redis.GetDatabase()

        Console.WriteLine("--- 06 Graph Processing ---")
        Dim key As String = "friends"

        ' GRAPH.ADD
        db.Execute("GRAPH.ADD", key, "Tom", "Jerry")
        db.Execute("GRAPH.ADD", key, "Jerry", "Spike")

        ' GRAPH.BFS
        Console.WriteLine("Friends of Tom:")
        Dim result = db.Execute("GRAPH.BFS", key, "Tom", 2)
        Console.WriteLine(result.ToString())
    End Sub
End Module
