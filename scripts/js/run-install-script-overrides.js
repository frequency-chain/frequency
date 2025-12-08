const { execSync } = require('child_process');
const fs = require('fs');

try {
    const data = fs.readFileSync('./package-exceptions.json', 'utf8');
    const exceptions = JSON.parse(data);

    exceptions.forEach(({ package, script}) => {
        console.log(`Overriding '--ignore-scripts' for package '${package}' with script '${script || 'install'}'...`);
        execSync(`npm run ${script || 'install'}`, {
            cwd: `node_modules/${package}`,
            stdio: 'inherit'
        });
    });
} catch {}
