node {
	 properties([
    pipelineTriggers([
        issueCommentTrigger('.*release.*')
    ])
])

    stage('Build') {
     node("benchmark") {
        checkout scm
        sh 'mkdir -p /data/tmp && export TMPDIR=/data/tmp &&  export PATH="/data/.cargo/bin:$PATH" && ln -s /data/.cargo /home/ubuntu/.cargo && rustup install nightly && rustup default nightly && make benchmarks'
     }
    }
}
