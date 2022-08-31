node {
    stage('Build') {
     node("benchmark") {
        checkout scm
        sh 'make benchmarks'
     }
    }
}
