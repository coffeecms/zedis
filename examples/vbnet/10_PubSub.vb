Imports StackExchange.Redis
Imports System
Imports System.Threading

Module Program
    Sub Main()
        Dim redis = ConnectionMultiplexer.Connect("localhost")
        Dim sub = redis.GetSubscriber()

        Console.WriteLine("--- 10 Pub/Sub ---")

        sub.Subscribe("vb_channel", Sub(channel, message)
            Console.WriteLine($"[Sub] Received: {message}")
        End Sub)

        Console.WriteLine("Subscribed. Sending in 1s...")
        Thread.Sleep(1000)

        sub.Publish("vb_channel", "Hello from VB.NET")
        Thread.Sleep(500)

        sub.UnsubscribeAll()
        redis.Close()
    End Sub
End Module
