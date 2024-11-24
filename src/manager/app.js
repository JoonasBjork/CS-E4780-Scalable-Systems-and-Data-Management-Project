import { createClient } from "npm:redis@4.6.4";
import { serve } from "./deps.js";
import async from "npm:async";



//const delay = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
// manager id
const MANAGER_ID = crypto.randomUUID();
// the number of workers
const workerCount = Deno.env.get("WORKER_COUNT");

// connect to redis
const client = createClient({
    url: "redis://redis-streams:6379",
    pingInterval: 1000,
});

try {
    await client.connect();
    console.log("success to connect to redis");
} catch (error) {
    console.error("failure to connect to redis", error);
}

// change into number from id
const stringToNumber = (str) => {
    let sum = 0;
    for (let i = 0; i < str.length; i++) {
        sum += str.charCodeAt(i);
    }
    return sum;
};

// make to stream name
const streamName = (str) => {
    let id_number = stringToNumber(str) % workerCount + 1;
    return `s${id_number}`;
};

let iter = 0;

let stock_data
async.forever(
    async () => {
        try {
            // fetch data from simulater
            stock_data = await client.xReadGroup(
                "managers",
                MANAGER_ID,
                {
                    key: "ingress",
                    id: ">",
                },
                {
                    count: 10,
                    block: 5000,
                }
            );
        } catch (error) {
            try {
                await client.xGroupCreate("ingress", "managers", "0", {
                    MKSTREAM: true,
                });
                stock_data = await client.xReadGroup(
                    "managers",
                    MANAGER_ID,
                    {
                        key: "ingress",
                        id: ">",
                    },
                    {
                        count: 10,
                        block: 5000,
                    }
                );
            } catch (groupError) {
                console.error("Failed to create group or read messages", groupError);
                return; 
            }
        }
       
        if(!stock_data){
            return;     
        }
        // reading data from fetched data
        if (stock_data) {
            for (let i = 0; i < stock_data[0].messages.length; i++) {
                const message = stock_data[0].messages[i].message;
                const message_id = stock_data[0].messages[i].id;
                const { id, sectype, last, time, date } = message;

                const queue_name = streamName(id);
                // send data to worker
                try {
                    await client.xAdd(queue_name, "*", {
                        id,
                        sectype,
                        last,
                        time,
                        date,
                    });
                    // delete data from simulater
                    await client.xDel("ingress", message_id);
                } catch (error) {
                    console.error("Failed to scale", error);
                }
            }
            if (++iter % 100 === 0) {
                // gabage collect 
                gc();
                //await delay(20);
                console.log(`Iteration: ${iter}, Memory usage:`, Deno.memoryUsage());
            }
        }
        stock_data=null;
        
    },
    (err) => {
        console.error("An error occurred in async.forever loop", err);
    }
);