const Redis = require("ioredis");

async function run() {
    console.log("--- 10 Pub/Sub ---");

    const sub = new Redis({ host: "localhost", port: 6379 });
    const pub = new Redis({ host: "localhost", port: 6379 });

    await sub.subscribe("node_channel");
    console.log("Subscribed to 'node_channel'");

    sub.on("message", (channel, message) => {
        console.log(`[Sub] Received on ${channel}: ${message}`);
        if (message === "QUIT") {
            sub.disconnect();
            pub.disconnect();
        }
    });

    // Small delay to ensure subscription is active
    await new Promise(r => setTimeout(r, 100));

    // Publish
    console.log("[Pub] Sending Hello...");
    await pub.publish("node_channel", "Hello from Node.js");

    await new Promise(r => setTimeout(r, 100));
    console.log("[Pub] Sending QUIT...");
    await pub.publish("node_channel", "QUIT");
}

run().catch(console.error);
