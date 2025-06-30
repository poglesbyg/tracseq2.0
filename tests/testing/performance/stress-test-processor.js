// Artillery processor for TracSeq 2.0 stress tests

module.exports = {
  // Generate random data for requests
  generateSampleData: function(context, events, done) {
    context.vars.sampleName = `STRESS-${Math.random().toString(36).substring(7)}`;
    context.vars.patientId = `PAT-${Math.floor(Math.random() * 100000)}`;
    context.vars.volumeMl = Math.floor(Math.random() * 20) + 1;
    context.vars.collectionDate = new Date().toISOString();
    return done();
  },

  // Generate sequencing data
  generateSequencingData: function(context, events, done) {
    context.vars.sequencingRunName = `SEQ-${Date.now()}-${Math.floor(Math.random() * 1000)}`;
    context.vars.readLength = [75, 150, 300][Math.floor(Math.random() * 3)];
    context.vars.coverage = Math.floor(Math.random() * 70) + 30;
    return done();
  },

  // Set authentication header
  setAuthHeader: function(requestParams, context, ee, next) {
    if (context.vars.authToken) {
      requestParams.headers = requestParams.headers || {};
      requestParams.headers['Authorization'] = `Bearer ${context.vars.authToken}`;
    }
    return next();
  },

  // Check response and capture data
  checkResponse: function(requestParams, response, context, ee, next) {
    if (response.statusCode >= 200 && response.statusCode < 300) {
      // Capture any IDs or tokens from successful responses
      if (response.body && typeof response.body === 'object') {
        if (response.body.id) {
          context.vars.lastResourceId = response.body.id;
        }
        if (response.body.access_token) {
          context.vars.authToken = response.body.access_token;
        }
      }
    } else {
      // Log errors for debugging
      console.error(`Request failed with status ${response.statusCode}`);
    }
    return next();
  },

  // Generate bulk data
  generateBulkData: function(context, events, done) {
    const samples = [];
    const count = Math.floor(Math.random() * 50) + 10;
    
    for (let i = 0; i < count; i++) {
      samples.push({
        name: `BULK-${Math.random().toString(36).substring(7)}`,
        sample_type: ['blood', 'tissue', 'dna', 'rna'][Math.floor(Math.random() * 4)],
        volume_ml: Math.floor(Math.random() * 10) + 1,
        patient_id: `PAT-${Math.floor(Math.random() * 10000)}`
      });
    }
    
    context.vars.bulkSamples = samples;
    return done();
  },

  // Simulate think time
  thinkTime: function(context, events, done) {
    setTimeout(done, Math.floor(Math.random() * 2000) + 1000);
  }
};