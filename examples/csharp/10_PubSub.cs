using StackExchange.Redis;
using System;
using System.Threading;

class Program
{
    static void Main()
    {
        var redis = ConnectionMultiplexer.Connect("localhost");
        var sub = redis.GetSubscriber();

        Console.WriteLine("--- 10 Pub/Sub ---");

        sub.Subscribe("cs_channel", (channel, message) => {
            Console.WriteLine($"[Sub] Received: {message}");
        });

        Console.WriteLine("Subscribed. Publishing in 1s...");
        Thread.Sleep(1000);

        sub.Publish("cs_channel", "Hello from C#");
        Thread.Sleep(500);
        
        // Cleanup
        sub.UnsubscribeAll();
        redis.Close();
    }
}
