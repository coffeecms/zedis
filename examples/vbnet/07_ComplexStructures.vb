Imports StackExchange.Redis
Imports System

Module Program
    Sub Main(args As String())
        Dim redis = ConnectionMultiplexer.Connect("localhost")
        Dim db = redis.GetDatabase()

        Console.WriteLine("--- 07 Complex Structures ---")

        ' List
        db.ListRightPush("cart", "apple")
        db.ListRightPush("cart", "banana")
        Dim item = db.ListLeftPop("cart")
        Console.WriteLine($"Removed from cart: {item}")

        ' Set
        db.SetAdd("colors", "red")
        db.SetAdd("colors", "blue")
        Dim isRed As Boolean = db.SetContains("colors", "red")
        Console.WriteLine($"Contains red? {isRed}")

        ' Hash
        db.HashSet("config:ui", New HashEntry() {
            New HashEntry("theme", "dark"),
            New HashEntry("fontsize", "14px")
        })
        Dim theme = db.HashGet("config:ui", "theme")
        Console.WriteLine($"Theme: {theme}")
    End Sub
End Module
