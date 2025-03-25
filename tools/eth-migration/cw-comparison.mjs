import fs from "fs/promises";
import pg from "pg";

const { Client } = pg;

// Configuration
const dbConfig = {
	host: 'Host',     // Database host
	port: 5432,            // Database port
	database: 'database_name',   // Database name
	user: 'user_name', // Database username
	password: 'password' // Database password
};

const inputFilePath = 'file.txt';     // Path to your input file
const tableName = 'table_name';       // Your database table name
const columnName = 'column_name';     // Column to compare
const condition = 'some_condition'; 	 // Condition

async function compareValues(inputFilePath, dbConfig, tableName, columnName) {
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
compareValues(inputFilePath, dbConfig, tableName, columnName)
	.then(result => {
		// Optional: Do something with the results
		console.log('Comparison completed');
	})
	.catch(error => {
		console.error('Comparison failed:', error);
	});
