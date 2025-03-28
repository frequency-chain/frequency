import fs from "fs/promises";
import pg from "pg";

const { Client } = pg;

// Database host
const DB_HOST = process.env["DB_HOST"];
// Database port
const DB_PORT = process.env["DB_PORT"] || 5432;
// Database username
const DB_USERNAME = process.env["DB_USERNAME"];
// Database password
const DB_PASSWORD = process.env["DB_PASSWORD"];
// Database name
const DB_NAME = process.env["DB_NAME"];
// Your query db table name
const DB_TABLE_NAME = process.env["DB_TABLE"];
// Your DB column to compare
const DB_COLUMN_NAME = process.env["DB_COLUMN"];
// DB query condition
const DB_CONDITION = process.env["DB_CONDITION"];
// Path to your input file
const INPUT_FILE_LOCATION = process.env["INPUT_FILE"] || 'file.txt';

// Configuration
const dbConfig = {
	host: DB_HOST,
	port: DB_PORT,
	database: DB_NAME,
	user: DB_USERNAME,
	password: DB_PASSWORD,
};

async function compareValues(inputFilePath, dbConfig, tableName, columnName, condition) {
	// Read values from file
	const fileContent = await fs.readFile(inputFilePath, 'utf-8');
	const fileValues = new Set(
		fileContent
			.split('\n')
			.map(line => line.trim().replace(/^'|'$/g, ''))
			.filter(line => line) // Remove empty lines
	);

	// Connect to PostgreSQL
	const client = new Client(dbConfig);

	try {
		await client.connect();

		// Query database values
		const query = `SELECT ${columnName} FROM ${tableName} WHERE ${condition}`;
		const result = await client.query(query);

		const dbValues = new Set(
			result.rows.map(row => row[columnName])
		);

		// Find values only in database
		const dbOnlyValues = new Set(
			[...dbValues].filter(x => !fileValues.has(x))
		);

		// Find values only in file
		const fileOnlyValues = new Set(
			[...fileValues].filter(x => !dbValues.has(x))
		);

		// Output results
		console.log('Values in Database but not in File:');
		dbOnlyValues.forEach(value => console.log(value));

		console.log('\nValues in File but not in Database:');
		fileOnlyValues.forEach(value => console.log(value));

		return {
			dbOnlyValues,
			fileOnlyValues
		};

	} catch (error) {
		console.error('Error comparing values:', error);
	} finally {
		await client.end();
	}
}

// Run the comparison
compareValues(INPUT_FILE_LOCATION, dbConfig, DB_TABLE_NAME, DB_COLUMN_NAME, DB_CONDITION)
	.then(result => {
		// Optional: Do something with the results
		console.log('Comparison completed');
	})
	.catch(error => {
		console.error('Comparison failed:', error);
	});
