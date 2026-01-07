<?php
require 'vendor/autoload.php';

echo "--- 10 Pub/Sub ---\n";

// NOTE: PHP scripts are usually synchronous. 
// Standard pattern is running a separate script for subscriber loop.

$pid = pcntl_fork();

if ($pid == -1) {
    die("Could not fork");
} else if ($pid) {
    // Parent - Publisher
    sleep(1); // Wait for sub
    $pub = new Predis\Client();
    echo "[Pub] Sending...\n";
    $pub->publish("php_channel", "Hello from PHP");
    sleep(1);
    $pub->publish("php_channel", "QUIT");
    pcntl_wait($status); // Wait for child
} else {
    // Child - Subscriber
    $sub = new Predis\Client(['read_write_timeout' => -1]);
    $pubsub = $sub->pubSubLoop();
    $pubsub->subscribe("php_channel");

    foreach ($pubsub as $message) {
        if ($message->kind === 'message') {
            echo "[Sub] Received: {$message->payload}\n";
            if ($message->payload === 'QUIT') {
                $pubsub->unsubscribe();
                break;
            }
        }
    }
}
?>
