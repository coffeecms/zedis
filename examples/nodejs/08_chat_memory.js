const Redis = require("ioredis");

const redis = new Redis({ host: "localhost", port: 6379 });

// Mock Chatbot with Memory
async function main() {
    console.log("--- 08 Advanced: Chatbot Memory with Vector Search ---");

    const chatId = "chat:user:123";

    // 1. Store Chat History as Vectors
    // In a real app, you'd store prompt + response pairs
    const history = [
        "User: My name is Tien.",
        "User: I live in Vietnam.",
        "User: I like coding in Rust.",
    ];

    console.log("Storing chat history...");
    for (let i = 0; i < history.length; i++) {
        // Generate a unique ID for each message
        const msgId = `${chatId}:msg:${i}`;
        await redis.call("VADD.TEXT", msgId, history[i]);
    }

    // 2. Retrieval (Recall)
    const currentQuery = "What is my name?";
    console.log(`\nNew User Query: "${currentQuery}"`);
    console.log("Recalling relevant history...");

    const context = await redis.call("VSEARCH.TEXT", chatId, currentQuery, 1);
    console.log("Relevant Memory Found:", context);

    if (context.length > 0) {
        console.log(`\nBot: Based on your history, I know '${context[0]}'. So your name is Tien.`);
    }

    redis.disconnect();
}

main();
